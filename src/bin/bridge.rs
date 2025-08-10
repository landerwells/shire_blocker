use serde_json::Value;
use serde_json::json;
use shire_blocker::{recv_length_prefixed_message, send_length_prefixed_message};
use std::io::Write;
use std::io::{self, Read};
use std::os::unix::net::UnixStream;

// I could potentially refactor these out. I would be able to use the send and receive
// functions from the shire_blocker crate, but I want to keep this simple for now.
fn read_message() -> io::Result<Vec<u8>> {
    let mut length_buf = [0u8; 4];
    io::stdin().read_exact(&mut length_buf)?;
    let length = u32::from_le_bytes(length_buf);

    let mut message_buf = vec![0u8; length as usize];
    io::stdin().read_exact(&mut message_buf)?;

    Ok(message_buf)
}

fn write_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Read the message to send from somewhere (stdin or file)
    let message = read_message()?; // Assuming this returns Vec<u8>

    let mut stream = UnixStream::connect("/tmp/shire_bridge.sock")?;
    send_length_prefixed_message(&mut stream, &message)?;

    match recv_length_prefixed_message(&mut stream) {
        Ok(response) => {
            let response_str = String::from_utf8_lossy(&response);
            let v: Value = serde_json::from_str(&response_str).unwrap_or_else(|_| {
                eprintln!("Invalid JSON response from daemon.");
                json!({})
            });

            let allowed = v["allowed"].as_bool().unwrap_or(false);

            if allowed {
                eprintln!("URL is allowed");
                write_message(
                    &json!({
                        "status": "allowed",
                        "message": "This site is allowed"
                    })
                    .to_string(),
                )?;
            } else {
                eprintln!("URL is blocked");
                write_message(
                    &json!({
                        "status": "blocked",
                        "message": "This site is blocked"
                    })
                    .to_string(),
                )?;
            }
        }
        Err(e) => {
            eprintln!("Failed to communicate with daemon: {e}");
            write_message(
                &json!({
                    "status": "error",
                    "message": "Failed to communicate with daemon"
                })
                .to_string(),
            )?;
        }
    }

    Ok(())
}
