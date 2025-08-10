use std::io::{self, Write, Read};
use std::os::unix::net::UnixStream;

pub fn send_length_prefixed_message(
    stream: &mut UnixStream,
    message: &[u8],
) -> io::Result<()> {
    let length = (message.len() as u32).to_be_bytes();
    stream.write_all(&length)?;
    stream.write_all(message)?;
    Ok(())
}

pub fn recv_length_prefixed_message(stream: &mut UnixStream) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}
