use std::{collections::HashSet, io::Read};
use crate::config::Block;
use std::net::{TcpListener, TcpStream};

mod config;

fn main() {
    // When first starting up the daemon, the only blocks that will be 
    // enabled will be
    let config = config::parse_config().unwrap();

    let mut active_blocks: HashSet<Block> = HashSet::new();
    // populate active_blocks with the blocks that are set to be true on startup
    // Default strategy would be to disable all blocks?
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
        let url = handle_client(&mut stream.unwrap());
        // Still need to 
        let mut block_site = false;
        
        // Need more complex logic on whether the block should go through.
 
        // if is_blacklisted(active_blocks, url) && !is_whitelisted() {
        //     // Send a message back through the TCPListener
        //
        // }
        // Check if the URL is in the blacklist of any block
        if active_blocks.iter().any(|block| {
            block.blacklist.as_ref().is_some_and(|blacklist| {
                blacklist.iter().any(|b| url.contains(b))
            })
        }) {
            println!("Blocked URL: {}", url);
            // Here you would handle the blocking logic, e.g., sending a response
            // to the client or logging the blocked request.
        } else {
            println!("Allowed URL: {}", url);
            // Handle allowed URL logic here.
        }

    }
}

fn handle_client(stream: &mut TcpStream) -> String {
    let mut buffer = [0; 512];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                // Process the received data
                println!("Received {n} bytes");
                // handle_message(&mut stream, &buffer[..n]);
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {e}");
                break;
            }
        }
    }
    println!(" {}", String::from_utf8_lossy(&buffer));
    String::from_utf8_lossy(&buffer).to_string()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist() {
        // Need to generate a test file possibly, so that way I don't have to 
        // rely on having the correct config.

        let config = config::parse_config().unwrap();

        let blocks = config.blocks;

        let url = "https://www.youtube.com/";
        // Check if the URL is in the blacklist of any block
        assert!(
            blocks.iter().any(|block| {
                block.blacklist.as_ref().map_or(false, |blacklist| {
                    blacklist.iter().any(|b| url.contains(b))
                })
            }),
            "URL should be blocked"
        );
    }
}
