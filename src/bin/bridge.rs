use serde_json::json;
use std::io::Write;
use std::io::{self, Read};
use std::net::TcpStream;

fn read_message() -> io::Result<String> {
    let mut length_buf = [0u8; 4];
    io::stdin().read_exact(&mut length_buf)?;
    let length = u32::from_le_bytes(length_buf);

    let mut message_buf = vec![0u8; length as usize];
    io::stdin().read_exact(&mut message_buf)?;

    let message = String::from_utf8(message_buf).expect("Invalid UTF-8 message");
    Ok(message)
}

fn write_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

// {
//   "source": "bridge",
//   "type": "url-check",
//   "payload": { "url": "youtube.com" }
// }
// I think this is the only real function that needs to change, as it will be
// getting converted to use a Unix socket
fn send_to_daemon(message: &str) -> io::Result<bool> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    // Send length-prefixed message to daemon
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(bytes)?;
    stream.flush()?;

    // Read single byte response from daemon
    // This will no longer be a single byte response but that shouldn't make the
    // resulting output any different, there will just need to be more parsing.
    let mut response_buf = [0u8; 1];
    stream.read_exact(&mut response_buf)?;

    Ok(response_buf[0] == 1)
}

fn main() -> io::Result<()> {
    let json_string = read_message()?;
    eprintln!("Bridge received: {json_string}");

    match send_to_daemon(&json_string) {
        Ok(is_blocked) => {
            if is_blocked {
                eprintln!("URL is blocked");
                write_message(
                    &json!({
                        "status": "blocked",
                        "message": "This site is blocked"
                    })
                    .to_string(),
                )?;
            } else {
                eprintln!("URL is allowed");
                write_message(
                    &json!({
                        "status": "allowed",
                        "message": "This site is allowed"
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
