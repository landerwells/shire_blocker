use serde_json::{json, Value};
use shire_blocker::{recv_length_prefixed_message, send_length_prefixed_message};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// At some point I should make a bridge error log and then have everything write
// to that if there are problems

// fn read_browser_message() -> io::Result<Vec<u8>> {
//     let mut length_buf = [0u8; 4];
//     io::stdin().read_exact(&mut length_buf)?;
//     let length = u32::from_le_bytes(length_buf);
//
//     let mut message_buf = vec![0u8; length as usize];
//     io::stdin().read_exact(&mut message_buf)?;
//
//     Ok(message_buf)
// }

fn write_browser_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

fn connect_to_daemon() -> io::Result<UnixStream> {
    UnixStream::connect("/tmp/shire_bridge.sock")
}

// fn request_state_from_daemon(stream: &mut UnixStream) -> io::Result<ApplicationState> {
//     let request = json!({
//         "action": "get_state"
//     });
//
//     send_length_prefixed_message(stream, request.to_string().as_bytes())?;
//
//     match recv_length_prefixed_message(stream) {
//         Ok(response) => {
//             let response_str = String::from_utf8_lossy(&response);
//             let v: Value = serde_json::from_str(&response_str).unwrap_or_else(|_| {
//                 eprintln!("Invalid JSON response from daemon");
//                 json!({})
//             });
//
//             let mut active_blocks = Vec::new();
//             if let Some(blocks) = v["active_blocks"].as_array() {
//                 for block in blocks {
//                     let mut blacklist = None;
//                     let mut whitelist = None;
//
//                     if let Some(bl) = block["blacklist"].as_array() {
//                         blacklist = Some(bl.iter().filter_map(|s| s.as_str().map(String::from)).collect());
//                     }
//
//                     if let Some(wl) = block["whitelist"].as_array() {
//                         whitelist = Some(wl.iter().filter_map(|s| s.as_str().map(String::from)).collect());
//                     }
//
//                     active_blocks.push(BlockConfig {
//                         name: block["name"].as_str().unwrap_or("unknown").to_string(),
//                         blacklist,
//                         whitelist,
//                         locked_until: block["locked_until"].as_u64(),
//                     });
//                 }
//             }
//
//             Ok(ApplicationState {
//                 active_blocks,
//                 daemon_connected: true,
//             })
//         }
//         Err(e) => {
//             eprintln!("Failed to get state from daemon: {e}");
//             Ok(ApplicationState {
//                 active_blocks: Vec::new(),
//                 daemon_connected: false,
//             })
//         }
//     }
// }

// fn send_state_to_browser(state: &ApplicationState) -> io::Result<()> {
//     let state_message = json!({
//         "type": "state_update",
//         "state": {
//             "active_blocks": state.active_blocks.iter().map(|block| {
//                 json!({
//                     "name": block.name,
//                     "blacklist": block.blacklist,
//                     "whitelist": block.whitelist,
//                     "locked_until": block.locked_until
//                 })
//             }).collect::<Vec<_>>(),
//             "daemon_connected": state.daemon_connected
//         }
//     });
//
//     write_browser_message(&state_message.to_string())
// }

fn main() -> io::Result<()> {
    loop {
        match connect_to_daemon() {
            Ok(mut stream) => {
                if let Err(e) = message_loop(&mut stream) {
                    eprintln!("Message loop error: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to daemon: {e}");
                thread::sleep(Duration::from_secs(10));
            }
        }
    }
}


fn message_loop(stream: &mut UnixStream) -> std::io::Result<()> {
    loop {
        // Receive message from daemon
        match recv_length_prefixed_message(stream) {
            Ok(message_bytes) => {
                // Parse the JSON message
                let message_str = String::from_utf8_lossy(&message_bytes);
                match serde_json::from_str::<Value>(&message_str) {
                    Ok(json_message) => {
                        // Probably turn this into a match at some point to support
                        // not only "state_update" messages
                        if let Some(msg_type) = json_message.get("type") {
                            if msg_type == "state_update" {
                                // Forward the entire message to the browser extension
                                if let Err(e) = write_browser_message(&message_str) {
                                    eprintln!("Failed to forward message to browser: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON message from daemon: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to receive message from daemon: {e}");
                return Err(e);
            }
        }
    }
}
