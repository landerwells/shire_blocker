use crate::config::Block;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::{collections::HashSet, io::Read};
use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, Mutex};

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
    // let mut block_states: HashMap<String, BlockState> = HashMap::new();
    let mut block_states = Arc::new(Mutex::new(HashMap::new()));

    config.blocks.iter().for_each(|block| {
        let state = if matches!(block.active_by_default, Some(true)) {
            BlockState::Blocked
        } else {
            BlockState::Unblocked
        };
        // Lock the mutex and insert into the HashMap
        let mut map = block_states.lock().unwrap();
        map.insert(block.name.clone(), state);
    });

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            let active_blocks: HashSet<Block> = config.clone()
                .blocks
                .into_iter()
                .filter(|block| block.active_by_default.unwrap_or(false))
                .collect();
            match stream {
                Ok(mut stream) => {
                    handle_bridge_request(&mut stream, &active_blocks);
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let blocks = block_states.clone();
                thread::spawn(move || handle_cli_request(&mut stream, blocks));
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }

        // Need to join back with main thread and update the block states
        // And I think this just got a whole lot more complicated because I need
        // to pass the updated block states to the bridge thread
    }
}

fn handle_cli_request(stream: &mut UnixStream, blocks: Arc<Mutex<HashMap<String, BlockState>>>) {
    if let Some(json_str) = read_length_prefixed_message(stream) {
        let v: Value = serde_json::from_str(json_str.trim()).unwrap_or_else(|_| {
            eprintln!("Invalid JSON from CLI.");
            serde_json::json!({})
        });

        let mut map = blocks.lock().unwrap();

        match v["action"].as_str() {
            Some("list_blocks") => {
                println!("{blocks:?}");
                let response = serde_json::json!({ "blocks": *map });
                let response_str = response.to_string();
                let bytes = response_str.as_bytes();
                let len = bytes.len() as u32;
                stream.flush().unwrap();
                let _ = stream.write_all(&len.to_le_bytes());
                let _ = stream.write_all(bytes);
            }
            // I guess I could just have a toggle block action to simplify both
            // block start and block stop
            Some("start_block") => {
                // Could have multiple errors in this method, first the block might 
                // not exist, then
                let block_name = v["name"].as_str().unwrap().to_string();

                if let Some(state) = map.get_mut(&block_name) {
                    println!("Found block {block_name}");
                    *state = BlockState::Blocked;
                    println!("{:?}", blocks);
                } else {
                    eprintln!("Block '{}' not found.", block_name);
                }
            }
            _ => eprintln!("Unknown action in CLI request."),
        }

        // let _ = stream.write_all(&[if allowed { 0 } else { 1 }]);
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

fn is_blacklisted(active_blocks: &HashSet<Block>, url: &str) -> bool {
    let url = remove_http_www(url.to_string());
    active_blocks.iter().any(|block| {
        block
            .blacklist
            .as_ref()
            .is_some_and(|blacklist| blacklist.iter().any(|b| url.starts_with(b)))
    })
}

fn is_whitelisted(active_blocks: &HashSet<Block>, url: &str) -> bool {
    let url = remove_http_www(url.to_string());

    active_blocks.iter().any(|block| {
        block
            .whitelist
            .as_ref()
            .is_some_and(|whitelist| whitelist.iter().any(|w| {
                let w_string = w.trim_end_matches('*').to_string();
                url.starts_with(&w_string)
            }))
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

fn handle_bridge_request(stream: &mut UnixStream, active_blocks: &HashSet<Block>) {
    if let Some(json_str) = read_length_prefixed_message(stream) {
        let v: Value = serde_json::from_str(json_str.trim()).unwrap_or_else(|_| {
            eprintln!("Invalid JSON from bridge.");
            serde_json::json!({})
        });

        let url = v["url"].as_str().unwrap_or("").to_string();
        let url = remove_http_www(url);

        let allowed = !is_blacklisted(active_blocks, &url) || is_whitelisted(active_blocks, &url);
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

    // Return value from setup
    fn setup_active_blocks() -> HashSet<Block> {
        let config = config::parse_config().unwrap();

        config
            .blocks
            .into_iter()
            .filter(|block| block.active_by_default.unwrap_or(false))
            .collect()
    }

    #[test]
    fn test_blacklist() {
        let active_blocks = setup_active_blocks();

        let url = "https://www.youtube.com/";
        assert!(is_blacklisted(&active_blocks, url));
    }

    #[test]
    fn test_whitelist() {
        let active_blocks = setup_active_blocks();

        let url = "https://www.youtube.com/results?search_query=sylvan+franklin";
        assert!(is_whitelisted(&active_blocks, url));
    }

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
