// use once_cell::sync::Lazy;
use crate::config::Block;
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use std::{collections::HashSet, io::Read};
use std::io::prelude::*;
use url::Url;

mod config;

fn main() {
    let config = config::parse_config().unwrap();
    let mut active_blocks: HashSet<Block> = HashSet::new();

    for block in config.blocks {
        if let Some(true) = block.active_by_default {
            active_blocks.insert(block);
        }
    }
    println!("Active blocks: {:?}", active_blocks);

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // At this point, there are three things that we will end up waiting on,
    // or need to figure out. Waiting on config to change, waiting on input from
    // the CLI, or waiting on input from browser.

    for stream in listener.incoming() {
        let mut stream = stream.unwrap(); // Unwrap once and reuse `stream`

        let json_str = handle_client(&mut stream).trim_matches('\0').to_string();
        let v: Value = serde_json::from_str(json_str.trim()).unwrap();

        let url = Url::parse(v["url"].as_str().unwrap_or("")).unwrap();
        let mut url_string = url.as_str().to_string();

        println!("Received URL: {}", url_string);

        url_string = remove_http_www(url_string);

        if is_blacklisted(&active_blocks, &url_string) {
            // Send a message back through the TCP
            println!("Blocked URL: {}", url_string);
            // Send a 1 to indicate the URL is blocked
            let _ = stream.write_all(&[1]);
        } else {
            println!("Allowed URL: {}", url_string);
            // Send a 0 to indicate the URL is allowed
            let _ = stream.write_all(&[0]);
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

fn is_blacklisted(active_blocks: &HashSet<Block>, url: &str) -> bool {
    active_blocks.iter().any(|block| {
        block.blacklist.as_ref().is_some_and(|blacklist| {
            blacklist.iter().any(|b| url.starts_with(b))
        })
    })
}

fn handle_client(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                println!("Received {n} bytes");
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {e}");
                break;
            }
        }
    }
    String::from_utf8_lossy(&buffer).to_string()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist() {
        let config = config::parse_config().unwrap();

        let blocks = config.blocks;

        let url = "https://www.youtube.com/";
        // Check if the URL is in the blacklist of any block
        assert!(
            blocks.iter().any(|block| {
                block.blacklist.as_ref().is_some_and(|blacklist| {
                    blacklist.iter().any(|b| url.contains(b))
                })
            }),
            "URL should be blocked"
        );
    }

    #[test]
    fn test_parse_json() {
        let json_str = r#"{"url":"https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"}"#;

        let v: Value = serde_json::from_str(json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {}", json_str);
            Value::Null
        });
        assert_eq!(v["url"], "https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust");

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
