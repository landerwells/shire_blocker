use serde_json::{json, Value};
use shire_blocker::{recv_length_prefixed_message, send_length_prefixed_message};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// At some point I should make a bridge error log and then have everything write
// to that if there are problems

fn connect_to_daemon() -> io::Result<UnixStream> {
    UnixStream::connect("/tmp/shire_bridge.sock")
}

fn read_browser_message() -> io::Result<Vec<u8>> {
    let mut length_buf = [0u8; 4];
    io::stdin().read_exact(&mut length_buf)?;
    let length = u32::from_le_bytes(length_buf);

    let mut message_buf = vec![0u8; length as usize];
    io::stdin().read_exact(&mut message_buf)?;

    Ok(message_buf)
}

fn write_browser_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}


// Have to be extremely careful in this file, cannot randomly println anywhere
// must switch to printing to a log
fn main() -> io::Result<()> {
    let message = read_browser_message();

    loop {
        match connect_to_daemon() {
            Ok(mut stream) => {
                // Could be a good idea to send a message to the js confirming the connection
                let request = json!({
                    "action": "get_state"
                });

                send_length_prefixed_message(&mut stream, request.to_string().as_bytes())?;

                // get the state from the daemon,
                match recv_length_prefixed_message(&mut stream) {
                    Ok(response) => {
                        let response_str = String::from_utf8_lossy(&response);
                        let v: Value = serde_json::from_str(&response_str).unwrap_or_else(|_| {
                            eprintln!("Invalid JSON response from daemon");
                            json!({})
                        });

                        write_browser_message(&response_str)?;

                    }
                    Err(e) => {
                        let response = json!({
                            "type": "error",
                            "message": "Failed to get state from daemon"
                        });
                    }
                }
                

            }
            Err(_) => {
                // eprintln!("Failed to connect to daemon: {e}");
                let response = json!({
                    "type": "error",
                    "message": "Failed to connect to daemon. Retrying..."
                });

                write_browser_message(&response.to_string())?;
                thread::sleep(Duration::from_secs(1)); // Retry after a delay
            }
        }
    }
}


// fn message_loop(stream: &mut UnixStream) -> std::io::Result<()> {
//     loop {
//         // Receive message from daemon
//         match recv_length_prefixed_message(stream) {
//             Ok(message_bytes) => {
//                 // Parse the JSON message
//                 let message_str = String::from_utf8_lossy(&message_bytes);
//                 match serde_json::from_str::<Value>(&message_str) {
//                     Ok(json_message) => {
//                         // Probably turn this into a match at some point to support
//                         // not only "state_update" messages
//                         if let Some(msg_type) = json_message.get("type") {
//                             if msg_type == "state_update" {
//                                 // Forward the entire message to the browser extension
//                                 if let Err(e) = write_browser_message(&message_str) {
//                                     eprintln!("Failed to forward message to browser: {e}");
//                                 }
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Failed to parse JSON message from daemon: {e}");
//                     }
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Failed to receive message from daemon: {e}");
//                 return Err(e);
//             }
//         }
//     }
// }

