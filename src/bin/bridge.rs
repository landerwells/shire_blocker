// use std::convert::TryInto;
// use std::io::{self, Read, Write};

// Reads a message from stdin as defined by the Native Messaging protocol.
// fn read_message() -> io::Result<String> {
//     let mut length_buf = [0u8; 4];
//     io::stdin().read_exact(&mut length_buf)?;
//     let length = u32::from_le_bytes(length_buf);
//
//     let mut message_buf = vec![0u8; length as usize];
//     io::stdin().read_exact(&mut message_buf)?;
//
//     let message = String::from_utf8(message_buf).expect("Invalid UTF-8 message");
//     Ok(message)
// }

/// Writes a message to stdout in the Native Messaging protocol format.
fn write_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

use std::io::Write;
use std::io::{self, Read};
use std::net::TcpStream;

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

    // Send to daemon via TCP
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    stream.write_all(json_sring.as_bytes())?;

    let youtube = "https://www.youtube.com/";

    if &json_sring[4..] == youtube {
        // Send message back to the application???? Using stdout

    }

    // Get message back from daemon,
    

    // Write a message back to the extension

    Ok(())
}
