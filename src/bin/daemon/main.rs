use crate::config::Block;
use serde_json::Value;
use std::fs;
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::{collections::HashSet, io::Read};
use std::thread;

mod config;

const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

// For testing purposes, I think it would be beneficial to have a way to 
// pass a the configuration to the main function. This would allow us to 
// easily test different configurations without having to read from a file.

// I like the idea of a single app state that I can pass into the cli or 
// bridge thread. 
fn main() {
    // Maybe think of putting the config parsing in a separate function named
    // initialize_config or something similar. That way we can hotload the
    // config if we want to.
    let config = config::parse_config().unwrap();
    let mut active_blocks: HashSet<Block> = HashSet::new();

    for block in config.blocks {
        if let Some(true) = block.active_by_default {
            active_blocks.insert(block);
        }
    }
    println!("Active blocks: {active_blocks:?}");

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    let active_blocks_clone = active_blocks.clone();
    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    handle_bridge_request(&mut stream, &active_blocks_clone);
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    // I think at least for the cli I will need to pass all of the blocks
    // instead of just the active ones. I essentially need to print the entire
    // configuration to the user.
    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let blocks = active_blocks.clone();
                thread::spawn(move || handle_cli_request(&mut stream, &blocks));
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn handle_cli_request(stream: &mut UnixStream, active_blocks: &HashSet<Block>) {
    if let Some(json_str) = read_length_prefixed_message(stream) {
        let v: Value = serde_json::from_str(json_str.trim()).unwrap_or_else(|_| {
            eprintln!("Invalid JSON from CLI.");
            serde_json::json!({})
        });

        match v["action"].as_str() {
            Some("list_blocks") => {
                let blocks: Vec<String> = active_blocks
                    .iter()
                    .map(|block| block.name.clone())
                    .collect();
                let response = serde_json::json!({ "blocks": blocks });
                let response_str = response.to_string();
                let bytes = response_str.as_bytes();
                let len = bytes.len() as u32;
                let _ = stream.write_all(&len.to_le_bytes());
                let _ = stream.write_all(bytes);
            }
            // Some("toggle_block") => {
            //     if let Some(name) = v["name"].as_str() {
            //         if let Some(block) = active_blocks.iter().find(|b| b.name == name) {
            //             // Toggle the block's active state
            //             if active_blocks.contains(block) {
            //                 active_blocks.remove(block);
            //                 println!("Block '{}' deactivated.", name);
            //             } else {
            //                 active_blocks.insert(block.clone());
            //                 println!("Block '{}' activated.", name);
            //             }
            //         } else {
            //             eprintln!("Block '{}' not found.", name);
            //         }
            //     } else {
            //         eprintln!("No block name provided.");
            //     }
            // }
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
