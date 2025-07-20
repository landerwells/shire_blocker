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
//
// /// Writes a message to stdout in the Native Messaging protocol format.
// fn write_message(message: &str) -> io::Result<()> {
//     let bytes = message.as_bytes();
//     let len = bytes.len() as u32;
//     io::stdout().write_all(&len.to_le_bytes())?;
//     io::stdout().write_all(bytes)?;
//     io::stdout().flush()?;
//     Ok(())
// }
//
// fn main() -> io::Result<()> {
//     loop {
//         let message = match read_message() {
//             Ok(msg) => msg,
//             Err(_) => break, // Stop if there's an error or browser closes stdin
//         };
//
//         println!("Got message from browser: {}", message);
//
//         // Do something — e.g. parse JSON, forward to daemon, etc.
//         let response = r#"{"status":"ok"}"#;
//         write_message(response)?;
//     }
//
//     Ok(())
// }

use std::net::TcpStream;
use std::io::Write;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();
    stream.write_all(b"Hello from client").unwrap();
}
