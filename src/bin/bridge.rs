use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;
use shire_blocker::*;

fn write_browser_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Tell the browser extension the bridge process has started.
    write_browser_message(r#"{"status":"starting"}"#)?;

    let mut backoff = Duration::from_millis(100);
    const MAX_BACKOFF: Duration = Duration::from_secs(30);

    loop {
        match UnixStream::connect(BRIDGE_SOCKET_PATH) {
            Ok(mut stream) => {
                backoff = Duration::from_millis(100); // reset backoff on successful connect
                write_browser_message(r#"{"status":"connected"}"#)?;

                // Relay daemon state updates to the browser. The daemon pushes
                // state proactively on connect and after every block change.
                loop {
                    match recv_length_prefixed_message(&mut stream) {
                        Ok(response) => {
                            write_browser_message(&String::from_utf8_lossy(&response))?;
                        }
                        Err(_) => {
                            write_browser_message(r#"{"status":"disconnected"}"#)?;
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                thread::sleep(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        }
    }
}
