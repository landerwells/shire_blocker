use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use shire_blocker::recv_length_prefixed_message;
use shire_blocker::send_length_prefixed_message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::config::*;


const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum BlockState {
    Unblocked,
    Blocked,
    BlockedWithLock,
}

pub fn start_daemon() {
    let config = parse_config().unwrap();
    println!("{:?}", config);
    let block_states = Arc::new(Mutex::new(HashMap::<Block, BlockState>::new()));

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

    let blocks = Arc::clone(&block_states);
    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let _ = handle_bridge_request(&mut stream, &blocks.lock().unwrap().clone());
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let blocks = Arc::clone(&block_states);

                thread::spawn(move || {
                    handle_cli_request(&mut stream, blocks.clone());
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
    // Need to have better error handling here
    let response = recv_length_prefixed_message(stream).unwrap();
    let response_str = String::from_utf8_lossy(&response);

    let v: Value = serde_json::from_str(response_str.trim()).unwrap_or_else(|_| {
        eprintln!("Invalid JSON from CLI.");
        serde_json::json!({})
    });

    let mut map = blocks.lock().unwrap();

    match v["action"].as_str() {
        Some("list_blocks") => {

            let string_map: HashMap<String, BlockState> = map
                .iter()
                .map(|(block, state)| (block.name.clone(), *state))
                .collect();

            let message = serde_json::json!({ "blocks": string_map }).to_string().into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        // I could even make a function purely for changing the state of the blocks
        Some("start_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();
            if let Some(state) = map.iter_mut().find_map(|(b, state)| {
                if b.name == block_name { Some(state) } else { None }
            }) {
                *state = BlockState::Blocked;
            } else {
                eprintln!("Block '{block_name}' not found.");
            }

            let message = serde_json::json!({ "status": "started", "block": block_name }).to_string().into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        Some("stop_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();

            // Use iter_mut and find_map just like in start_block to avoid cloning keys
            if let Some(state) = map.iter_mut().find_map(|(b, state)| {
                if b.name == block_name { Some(state) } else { None }
            }) {
                *state = BlockState::Unblocked;
            } else {
                eprintln!("Block '{block_name}' not found.");
            }

            let message = serde_json::json!({ "status": "stopped", "block": block_name }).to_string().into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        _ => eprintln!("Unknown action in CLI request."),
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

fn handle_bridge_request(
    stream: &mut UnixStream,
    blocks: &HashMap<Block, BlockState>,
) -> io::Result<()> {
    // Get black/white lists
    let blacklist = get_blacklist(blocks);
    let whitelist = get_whitelist(blocks);

    // Receive length-prefixed JSON request
    let raw_request = recv_length_prefixed_message(stream)?;
    let request_str = String::from_utf8_lossy(&raw_request);
    println!("Bridge request: {}", request_str);

    let v: Value = serde_json::from_str(request_str.trim()).unwrap_or_else(|_| {
        eprintln!("Invalid JSON from bridge.");
        json!({})
    });

    let url = v["url"].as_str().unwrap_or("").to_string();
    let url = remove_http_www(url);

    println!("Checking URL: {}", url);

    // Check if allowed or blocked
    let allowed = !is_blacklisted(&blacklist, &url) || is_whitelisted(&whitelist, &url);

    println!(
        "{} URL from bridge: {}",
        if allowed { "Allowed" } else { "Blocked" },
        url
    );

    // Send JSON response with "allowed" key
    let response_json = json!({ "allowed": allowed });
    let response_bytes = response_json.to_string().into_bytes();
    send_length_prefixed_message(stream, &response_bytes)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json_str =
            r#"{"url":"https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"}"#;

        let v: Value = serde_json::from_str(json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {json_str}");
            Value::Null
        });
        assert_eq!(
            v["url"],
            "https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"
        );

        // Test with an invalid JSON string
        let invalid_json_str = r#"{"url": "https://www.example.com""#;
        let v_invalid: Value = serde_json::from_str(invalid_json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {invalid_json_str}");
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
