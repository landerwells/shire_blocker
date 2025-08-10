use shire_blocker::recv_length_prefixed_message;
use shire_blocker::send_length_prefixed_message;
use serde_json::Value;
use serde_json::json;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

// ANSI color escape codes
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

pub fn list_blocks(stream: &mut UnixStream) -> io::Result<()> {
    let message = json!({
        "action": "list_blocks"
    })
    .to_string().into_bytes();

    send_length_prefixed_message(stream, &message);

    let bytes: Vec<u8> = recv_length_prefixed_message(stream)?;
    let response = String::from_utf8(bytes).unwrap();

    let v: Value = serde_json::from_str(&response).expect("Invalid JSON");

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


// TODO: I think all three of these functions could be condensed down to one??
pub fn start_block(stream: &mut UnixStream, name: String, lock: Option<String>) -> io::Result<()> {
    let message = json!({
        "action": "start_block",
        "name": name,
        "lock": lock
    })
    .to_string().into_bytes();

    send_length_prefixed_message(stream, &message)?;

    let bytes: Vec<u8> = recv_length_prefixed_message(stream)?; // propagate I/O error
    let response = String::from_utf8(bytes).unwrap();

    println!("Response: {response}");

    Ok(())
}

pub fn stop_block(stream: &mut UnixStream, name: String) -> io::Result<()> {
    let message = json!({
        "action": "stop_block",
        "name": name
    }).to_string().into_bytes();

    send_length_prefixed_message(stream, &message)?;

    let bytes: Vec<u8> = recv_length_prefixed_message(stream)?; // propagate I/O error
    let response = String::from_utf8(bytes).unwrap();

    println!("Response: {response}");
    Ok(())
}

pub fn lock_block(stream: &mut UnixStream, name: String, lock: String) -> io::Result<()> {
    let message = json!({
        "action": "lock_block",
        "name": name,
        "lock": lock
    })
    .to_string().into_bytes();

    send_length_prefixed_message(stream, &message)?;

    let bytes: Vec<u8> = recv_length_prefixed_message(stream)?; // propagate I/O error
    let response = String::from_utf8(bytes).unwrap();

    println!("Response: {response}");
    Ok(())
}
