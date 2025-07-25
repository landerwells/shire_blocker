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

fn main() -> io::Result<()> {
    let json_sring = read_message()?;

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    stream.write_all(json_sring.as_bytes())?;

    // let youtube = "https://www.youtube.com/";

    // if &json_sring[4..] == youtube {
    //     // Send message back to the application???? Using stdout
    // }

    // Get message back from daemon,
    // let mut response_buf = vec![0u8; 4];
    // stream.read_exact(&mut response_buf)?;
    // let response_len = u32::from_le_bytes(response_buf);

    // Write a message back to the extension
    let response_message = "Message received and processed";
    write_message(response_message)?;

    Ok(())
}
