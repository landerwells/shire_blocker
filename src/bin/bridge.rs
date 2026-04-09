use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;
use shire_blocker::*;

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


fn main() -> io::Result<()> {
    let mut connected = false;
    write_browser_message(r#"{"status":"connected"}"#)?;

    loop {
        match UnixStream::connect(BRIDGE_SOCKET_PATH) {
            Ok(mut stream) => {
                if !connected {
                    // connected = true;
                    write_browser_message(r#"{"status":"connected"}"#)?;
                    
                    // ask daemon for state
                    let request = r#"{"action":"get_state"}"#;
                    send_length_prefixed_message(&mut stream, request.as_bytes())?;
                }

                // relay daemon → browser
                loop {
                    match recv_length_prefixed_message(&mut stream) {
                        Ok(response) => {
                            let response_str = String::from_utf8_lossy(&response);
                            write_browser_message(&response_str)?;
                        }
                        Err(_) => {
                            connected = false;
                            write_browser_message(r#"{"status":"disconnected"}"#)?;
                            break; // drop inner loop, reconnect
                        }
                    }
                }
            }
            Err(_) => {
                if connected {
                    connected = false;
                    write_browser_message(r#"{"status":"disconnected"}"#)?;
                }
                thread::sleep(Duration::from_secs(1)); // retry later
            }
        }
    }
}
