use std::io::Write;
use std::io::{self, Read};
use std::net::TcpStream;
use serde_json::json;


/// Writes a message to stdout in the Native Messaging protocol format.
fn write_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Read message in from stdin from extension
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // Read the 4-byte length prefix
    let mut len_buf = [0u8; 4];
    handle.read_exact(&mut len_buf)?;

    let msg_len = u32::from_le_bytes(len_buf);
    let mut msg_buf = vec![0u8; msg_len as usize];
    handle.read_exact(&mut msg_buf)?;

    let json_sring = String::from_utf8(msg_buf).unwrap();
    // 

    // Send to daemon via TCP
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    stream.write_all(json_sring.as_bytes())?;

    let youtube = "https://www.youtube.com/";

    if &json_sring[4..] == youtube {
        // Send message back to the application???? Using stdout

    }
    let response_json = json!({
        "status": "ok",
        "message": "Message received and processed"
    });

    // Get message back from daemon,
    // let mut response_buf = vec![0u8; 4];
    // stream.read_exact(&mut response_buf)?;
    // let response_len = u32::from_le_bytes(response_buf);

    // Write a message back to the extension
    write_message(&response_json.to_string())?;

    Ok(())
}
