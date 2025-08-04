use serde_json::json;
use std::os::unix::net::UnixStream;
use std::io::{self, Read, Write};

pub fn list_available_blocks(cli_sock: &mut UnixStream) -> io::Result<()> {

    let message = json!({
        "action": "list_blocks"
    })
    .to_string();

    // Send the message to the daemon
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    cli_sock.write_all(&len.to_le_bytes())?;
    cli_sock.write_all(bytes)?;
    cli_sock.flush()?;

    // Read the response from the daemon
    let mut response_buf = [0u8; 1024];
    let bytes_read = cli_sock.read(&mut response_buf)?;
    
    if bytes_read == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "No response from daemon"));
    }

    let response_str = String::from_utf8_lossy(&response_buf[..bytes_read]);
    // Need to parse the response as JSON to handle it properly
    println!("Available blocks: {}", response_str);
    
    Ok(())
}
