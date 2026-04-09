use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

pub const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
pub const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

pub fn send_length_prefixed_message(stream: &mut UnixStream, message: &[u8]) -> io::Result<()> {
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

pub fn log_to_file(path: &str, msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    writeln!(file, "{msg}").unwrap();
}
