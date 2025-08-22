use serde_json::{Value, json};
use shire_blocker::{recv_length_prefixed_message, send_length_prefixed_message};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
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
// fn main() -> io::Result<()> {
//     let message = read_browser_message();
//
//     loop {
//         match connect_to_daemon() {
//             Ok(mut stream) => {
//                 // Could be a good idea to send a message to the js confirming the connection
//                 let request = json!({
//                     "action": "get_state"
//                 });
//
//                 send_length_prefixed_message(&mut stream, request.to_string().as_bytes())?;
//
//                 // get the state from the daemon,
//                 loop {
//                     match recv_length_prefixed_message(&mut stream) {
//                         Ok(response) => {
//                             let response_str = String::from_utf8_lossy(&response);
//                             let v: String = serde_json::from_str(&response_str).unwrap_or_else(|_| {
//                                 // eprintln!("Invalid JSON response from daemon");
//                                 json!({})
//                             }).to_string();
//
//                             write_browser_message(&v)?;
//                         }
//                         Err(e) => {
//                             let response = json!({
//                                 "type": "error",
//                                 "message": "Failed to get state from daemon"
//                             }).to_string();
//
//                             write_browser_message(&response)?;
//                             break;
//                         }
//                     }
//                 }
//             }
//             Err(_) => {
//                 // eprintln!("Failed to connect to daemon: {e}");
//                 let response = json!({
//                     "type": "error",
//                     "message": "Failed to connect to daemon. Retrying..."
//                 });
//
//                 write_browser_message(&response.to_string())?;
//                 thread::sleep(Duration::from_secs(1)); // Retry after a delay
//             }
//         }
//     }
// }

// use std::net::Stream;
// use std::thread;
// use std::time::Duration;

fn main() {
    let addr = "/tmp/shire_bridge.sock"; // your daemon socket path
    let mut connected: Option<bool> = None; // None = unknown state

    loop {
        match UnixStream::connect(addr) {
            Ok(stream) => {
                if connected != Some(true) {
                    println!("Connected to daemon at {}", addr);
                    connected = Some(true);
                }

                // Hold the connection for a short time and then check if it's still valid
                thread::sleep(Duration::from_secs(2));

                // Check if the socket is still open by calling peer_addr()
                if stream.peer_addr().is_err() {
                    if connected != Some(false) {
                        println!("Disconnected from daemon");
                        connected = Some(false);
                    }
                }

                // Drop stream so next loop can reconnect if needed
                drop(stream);
            }
            Err(_) => {
                if connected != Some(false) {
                    println!("Disconnected from daemon");
                    connected = Some(false);
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
