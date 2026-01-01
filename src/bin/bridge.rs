use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;

// fn connect_to_daemon() -> io::Result<UnixStream> {
//     UnixStream::connect("/tmp/shire_bridge.sock")
// }
//
// fn read_browser_message() -> io::Result<Vec<u8>> {
//     let mut length_buf = [0u8; 4];
//     io::stdin().read_exact(&mut length_buf)?;
//     let length = u32::from_le_bytes(length_buf);
//
//     let mut message_buf = vec![0u8; length as usize];
//     io::stdin().read_exact(&mut message_buf)?;
//
//     Ok(message_buf)
// }
//
// fn write_browser_message(message: &str) -> io::Result<()> {
//     let bytes = message.as_bytes();
//     let len = bytes.len() as u32;
//     io::stdout().write_all(&len.to_le_bytes())?;
//     io::stdout().write_all(bytes)?;
//     io::stdout().flush()?;
//     Ok(())
// }

fn main() {
    let addr = "/tmp/shire_bridge.sock"; // your daemon socket path
    let mut connected: Option<bool> = None; // None = unknown state

    loop {
        match UnixStream::connect(addr) {
            Ok(stream) => {
                if connected != Some(true) {
                    println!("Connected to daemon at {}", addr);
                    connected = Some(true);
                }

                // Hold the connection for a short time and then check if it's still valid
                thread::sleep(Duration::from_secs(2));

                // Check if the socket is still open by calling peer_addr()
                if stream.peer_addr().is_err() && connected != Some(false) {
                    println!("Disconnected from daemon");
                    connected = Some(false);
                }

                // Drop stream so next loop can reconnect if needed
                drop(stream);
            }
            Err(_) => {
                if connected != Some(false) {
                    println!("Disconnected from daemon");
                    connected = Some(false);
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
