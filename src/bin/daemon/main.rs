use crate::config::Block;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{collections::HashSet, io::Read};

mod config;

const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum BlockState {
    Unblocked,
    Blocked,
    BlockedWithLock,
}

fn main() {
    let config = config::parse_config().unwrap();
    let block_states = Arc::new(Mutex::new(HashMap::<Block, BlockState>::new()));
    let (tx, rx) = mpsc::channel::<(HashSet<String>, HashSet<String>)>();

    config.blocks.iter().for_each(|block| {
        let state = if matches!(block.active_by_default, Some(true)) {
            BlockState::Blocked
        } else {
            BlockState::Unblocked
        };

        let mut map = block_states.lock().unwrap();
        map.insert(block.clone(), state);
    });

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    // Need to pass blacklist and whitelist to the bridge thread

    let mut blacklist = get_blacklist(&block_states.lock().unwrap());
    let mut whitelist = get_whitelist(&block_states.lock().unwrap());

    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // Check for updates without blocking
                    if let Ok((new_blacklist, new_whitelist)) = rx.try_recv() {
                        blacklist = new_blacklist;
                        whitelist = new_whitelist;
                    }

                    handle_bridge_request(&mut stream, blacklist.clone(), whitelist.clone());
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let blocks = Arc::clone(&block_states);
                let tx = tx.clone();

                thread::spawn(move || {
                    handle_cli_request(&mut stream, blocks.clone());

                    let map = blocks.lock().unwrap();
                    let blacklist: HashSet<String> = get_blacklist(&map);
                    let whitelist: HashSet<String> = get_whitelist(&map);

                    if tx.send((blacklist, whitelist)).is_err() {
                        eprintln!("Bridge thread has shut down");
                    }
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn get_blacklist(
    blocks: &HashMap<Block, BlockState>,
) -> HashSet<String> {
    blocks
        .iter()
        .filter_map(|(block, state)| {
            if *state == BlockState::Blocked || *state == BlockState::BlockedWithLock {
                block.blacklist.clone()
            } else {
                None
            }
        })
    .flatten()
        .collect()
}

fn get_whitelist(
    blocks: &HashMap<Block, BlockState>,
) -> HashSet<String> {
    blocks
        .iter()
        .filter_map(|(block, state)| {
            if *state == BlockState::Blocked || *state == BlockState::BlockedWithLock {
                block.whitelist.clone()
            } else {
                None
            }
        })
    .flatten()
        .collect()
}

fn handle_cli_request(stream: &mut UnixStream, blocks: Arc<Mutex<HashMap<Block, BlockState>>>) {
    if let Some(json_str) = read_length_prefixed_message(stream) {
        let v: Value = serde_json::from_str(json_str.trim()).unwrap_or_else(|_| {
            eprintln!("Invalid JSON from CLI.");
            serde_json::json!({})
        });

        let mut map: HashMap<Block, BlockState> = blocks.lock().unwrap().clone();

        match v["action"].as_str() {
            Some("list_blocks") => {

                let string_map: HashMap<String, BlockState> = map
                    .into_iter()
                    .map(|(block, state)| (block.name.clone(), state))
                    .collect();

                let response = serde_json::json!({ "blocks": string_map });
                let response_str = response.to_string();
                let bytes = response_str.as_bytes();
                let len = bytes.len() as u32;
                stream.flush().unwrap();
                let _ = stream.write_all(&len.to_le_bytes());
                let _ = stream.write_all(bytes);
            }
            Some("start_block") => {
                // Could have multiple errors in this method, first the block might 
                // not exist, then
                let block_name = v["name"].as_str().unwrap().to_string();

                // This is where I could return the error to the client 
                // that the block does not exits.
                let block = map.keys().find(|b| b.name == block_name).cloned().unwrap();

                if let Some(state) = map.get_mut(&block) {
                    *state = BlockState::Blocked;
                } else {
                    eprintln!("Block '{}' not found.", block_name);
                }
            }
            Some("stop_block") => {
                let block_name = v["name"].as_str().unwrap().to_string();

                let block = map.keys().find(|b| b.name == block_name).cloned().unwrap();

                if let Some(state) = map.get_mut(&block) {
                    *state = BlockState::Unblocked;
                } else {
                    eprintln!("Block '{}' not found.", block_name);
                }
            }
            _ => eprintln!("Unknown action in CLI request."),
        }
    }
}

fn remove_http_www(mut url_string: String) -> String {
    if url_string.starts_with("https://") {
        url_string = url_string.strip_prefix("https://").unwrap().to_string();
    }

    if url_string.starts_with("www.") {
        url_string = url_string.strip_prefix("www.").unwrap().to_string();
    }

    url_string
}

fn is_blacklisted(blacklist: &HashSet<String>, url: &str) -> bool {
    let url = remove_http_www(url.to_string());
    blacklist.iter().any(|entry| url.starts_with(entry))
}

fn is_whitelisted(whitelist: &HashSet<String>, url: &str) -> bool {
    let url = remove_http_www(url.to_string());

    whitelist.iter().any(|pattern| {
        let prefix = pattern.trim_end_matches('*');
        url.starts_with(prefix)
    })
}

fn read_length_prefixed_message(stream: &mut UnixStream) -> Option<String> {
    // Read 4-byte length prefix
    let mut length_buf = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut length_buf) {
        eprintln!("Failed to read length: {e}");
        return None;
    }
    let length = u32::from_le_bytes(length_buf) as usize;

    // Read exactly that many bytes for the message
    let mut buffer = vec![0u8; length];
    if let Err(e) = stream.read_exact(&mut buffer) {
        eprintln!("Failed to read message: {e}");
        return None;
    }

    let message = String::from_utf8_lossy(&buffer).to_string();
    // println!("Received message: {}", message);
    Some(message)
}

fn handle_bridge_request(stream: &mut UnixStream, blacklist: HashSet<String>, whitelist: HashSet<String>) {
    println!("Received request from bridge...");
    if let Some(json_str) = read_length_prefixed_message(stream) {
        let v: Value = serde_json::from_str(json_str.trim()).unwrap_or_else(|_| {
            eprintln!("Invalid JSON from bridge.");
            serde_json::json!({})
        });

        let url = v["url"].as_str().unwrap_or("").to_string();
        let url = remove_http_www(url);

        println!("Checking URL: {}", url);
        let allowed = !is_blacklisted(&blacklist, &url) || is_whitelisted(&whitelist, &url);
        println!(
            "{} URL from bridge: {}",
            if allowed { "Allowed" } else { "Blocked" },
            url
        );

        let _ = stream.write_all(&[if allowed { 0 } else { 1 }]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fn setup_active_blocks() -> HashSet<Block> {
    //     let config = config::parse_config().unwrap();
    //
    //     config
    //         .blocks
    //         .into_iter()
    //         .filter(|block| block.active_by_default.unwrap_or(false))
    //         .collect()
    // }
    //
    // #[test]
    // fn test_blacklist() {
    //     let active_blocks = setup_active_blocks();
    //
    //     let url = "https://www.youtube.com/";
    //     assert!(is_blacklisted(&active_blocks, url));
    // }
    //
    // #[test]
    // fn test_whitelist() {
    //     let active_blocks = setup_active_blocks();
    //
    //     let url = "https://www.youtube.com/results?search_query=sylvan+franklin";
    //     assert!(is_whitelisted(&active_blocks, url));
    // }

    #[test]
    fn test_parse_json() {
        let json_str =
            r#"{"url":"https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"}"#;

        let v: Value = serde_json::from_str(json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {}", json_str);
            Value::Null
        });
        assert_eq!(
            v["url"],
            "https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"
        );

        // Test with an invalid JSON string
        let invalid_json_str = r#"{"url": "https://www.example.com""#;
        let v_invalid: Value = serde_json::from_str(invalid_json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {}", invalid_json_str);
            Value::Null
        });
        assert!(v_invalid.is_null());
    }

    #[test]
    fn test_remove_http_www() {
        let url_with_http = "https://www.example.com".to_string();
        let url_without_http = remove_http_www(url_with_http);
        assert_eq!(url_without_http, "example.com".to_string());

        let url_with_https = "https://example.com".to_string();
        let url_without_https = remove_http_www(url_with_https);
        assert_eq!(url_without_https, "example.com".to_string());

        let url_with_www = "www.example.com".to_string();
        let url_without_www = remove_http_www(url_with_www);
        assert_eq!(url_without_www, "example.com".to_string());
    }
}
