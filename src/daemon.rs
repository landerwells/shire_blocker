use crate::state::*;
use crate::config;
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

const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

pub fn start_daemon(config_path: Option<String>) {
    let config = config::parse_config(config_path).unwrap();

    let app_state: Arc<Mutex<ApplicationState>> = initialize_application_state(config.clone());

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    // Pass application state to bridge thread
    let bridge_app_state = Arc::clone(&app_state);
    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let _ = handle_bridge_request(&mut stream, Arc::clone(&bridge_app_state));
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    // How to structure this thread in order to not have empty waiting?
    thread::spawn(move || {
        // loop {
        // }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let cli_app_state = Arc::clone(&app_state);

                thread::spawn(move || {
                    handle_cli_request(&mut stream, cli_app_state);
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn handle_cli_request(stream: &mut UnixStream, app_state: Arc<Mutex<ApplicationState>>) {
    // Need to have better error handling here
    let response = recv_length_prefixed_message(stream).unwrap();
    let response_str = String::from_utf8_lossy(&response);

    let v: Value = serde_json::from_str(response_str.trim()).unwrap_or_else(|_| {
        eprintln!("Invalid JSON from CLI.");
        serde_json::json!({})
    });

    let mut app_state_guard = app_state.lock().unwrap();

    match v["action"].as_str() {
        Some("list_blocks") => {
            let string_map: HashMap<String, BlockState> = app_state_guard
                .blocks
                .iter()
                .map(|(name, block)| (name.clone(), block.block_state))
                .collect();

            let message = serde_json::json!({ "blocks": string_map })
                .to_string()
                .into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        // I could even make a function purely for changing the state of the blocks
        Some("start_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();
            update_block(&mut app_state_guard, &block_name, BlockState::Blocked);

            let message = serde_json::json!({ "status": "started", "block": block_name })
                .to_string()
                .into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        Some("stop_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();
            update_block(&mut app_state_guard, &block_name, BlockState::Unblocked);

            let message = serde_json::json!({ "status": "stopped", "block": block_name })
                .to_string()
                .into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        _ => eprintln!("Unknown action in CLI request."),
    }
}

fn handle_bridge_request(
    stream: &mut UnixStream,
    app_state: Arc<Mutex<ApplicationState>>,
) -> io::Result<()> {
    // Get black/white lists
    let app_state_guard = app_state.lock().unwrap();
    let blacklist = get_blacklist(&app_state_guard.blocks);
    let whitelist = get_whitelist(&app_state_guard.blocks);

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

// These functions could be refactored to avoid duplication.
fn get_blacklist(blocks: &HashMap<String, Block>) -> HashSet<String> {
    blocks
        .values()
        .filter_map(|block| {
            if block.block_state == BlockState::Blocked || block.block_state == BlockState::BlockedWithLock {
                block.blacklist.clone()
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

fn get_whitelist(blocks: &HashMap<String, Block>) -> HashSet<String> {
    blocks
        .values()
        .filter_map(|block| {
            if block.block_state == BlockState::Blocked || block.block_state == BlockState::BlockedWithLock {
                block.whitelist.clone()
            } else {
                None
            }
        })
        .flatten()
        .collect()
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

fn remove_http_www(mut url_string: String) -> String {
    if url_string.starts_with("https://") {
        url_string = url_string.strip_prefix("https://").unwrap().to_string();
    }

    if url_string.starts_with("www.") {
        url_string = url_string.strip_prefix("www.").unwrap().to_string();
    }

    url_string
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
