use serde_json::Value;
use serde_json::json;
use std::os::unix::net::UnixStream;
use std::io::{self, Read, Write};

// ANSI color escape codes
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

pub fn list_blocks(cli_sock: &mut UnixStream) -> io::Result<()> {
    let message = json!({
        "action": "list_blocks"
    })
    .to_string();

    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    cli_sock.write_all(&len.to_le_bytes())?;
    cli_sock.write_all(bytes)?;
    cli_sock.flush()?;

    // Read the length of the response
    let mut len_buf = [0u8; 4];
    cli_sock.read_exact(&mut len_buf)?;
    let response_len = u32::from_le_bytes(len_buf) as usize;

    // Now read exactly that many bytes
    let mut response_buf = vec![0u8; response_len];
    cli_sock.read_exact(&mut response_buf)?;

    let response_str = String::from_utf8_lossy(&response_buf);
    let v: Value = serde_json::from_str(&response_str).expect("Invalid JSON");

    // Ensure blocks is an object
    let blocks = match v["blocks"].as_object() {
        Some(obj) => obj,
        None => {
            eprintln!("Response format error: 'blocks' is not an object.");
            return Ok(());
        }
    };

    // Determine max width for alignment
    let name_width = blocks.keys().map(|k| k.len()).max().unwrap_or(10).max("Block Name".len());
    let status_width = "Status".len();

    // Print header
    println!(
        "{:<width1$}  {:<width2$}",
        "Block Name",
        "Status",
        width1 = name_width,
        width2 = status_width
    );
    println!(
        "{:-<width1$}  {:-<width2$}",
        "",
        "",
        width1 = name_width,
        width2 = status_width
    );

    // Print each block with color-coded status
    for (name, state_val) in blocks {
        let status_str = state_val.as_str().unwrap_or("Unknown");
        let colored_status = match status_str {
            "Blocked" => format!("{YELLOW}Blocked{RESET}"),
            "BlockedWithLock" | "Locked" => format!("{RED}Locked{RESET}"),
            "Unblocked" => format!("{GREEN}Unblocked{RESET}"),
            _ => status_str.to_string(),
        };

        println!("{:<width1$}  {}", name, colored_status, width1 = name_width);
    }

    Ok(())
}

pub fn start_block(cli_sock: &mut UnixStream, name: String, lock: Option<String>) -> io::Result<()> {
    let message = json!({
        "action": "start_block",
        "name": name,
        "lock": lock
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

    println!("Hello?");
    let response_str = String::from_utf8_lossy(&response_buf[..bytes_read]);
    println!("Response: {}", response_str);

    Ok(())
}
