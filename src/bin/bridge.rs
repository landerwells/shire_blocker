use serde_json::{Value, json};
use shire_blocker::{recv_length_prefixed_message, send_length_prefixed_message};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc};
use std::sync::mpsc::channel;

fn read_browser_message() -> io::Result<Vec<u8>> {
    let mut length_buf = [0u8; 4];
    io::stdin().read_exact(&mut length_buf)?;
    let length = u32::from_le_bytes(length_buf);

    let mut message_buf = vec![0u8; length as usize];
    io::stdin().read_exact(&mut message_buf)?;

    Ok(message_buf)
}

fn write_browser_message(message: &str) -> io::Result<()> {
    let bytes = message.as_bytes();
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(bytes)?;
    io::stdout().flush()?;
    Ok(())
}

enum Status {
    Disconnected,
    Connected,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "/tmp/shire_bridge.sock";
    let mut connected: Option<bool> = None; 

    // Shared flag to signal threads to exit
    // let running = Arc::new(AtomicBool::new(true));

    // --- Reading thread ---
    // let running_clone = Arc::clone(&running);
    let read_handle = thread::spawn(move || {
        while let Ok(read_stream) = rx.recv() {
            // while running_clone.load(Ordering::SeqCst) {
            // What happens when I close the connection while waiting right here?
            match recv_length_prefixed_message(&mut read_stream.try_clone().unwrap()) {
                Ok(msg) => {
                    write_browser_message(&String::from_utf8(msg).unwrap()).unwrap_or(());
                }
                Err(_) => ()
            }
        }
        });

    // --- Writing thread ---
    // let write_stream = stream.try_clone()?;
    let running_clone = Arc::clone(&running);
    let write_handle = thread::spawn(move || {
        while running_clone.load(Ordering::SeqCst) {
            let msg = read_browser_message().unwrap();
            let _ = send_length_prefixed_message(&mut write_stream.try_clone().unwrap(), &msg);
        }
    });

    loop {
        match UnixStream::connect(addr) {
            Ok(stream) => {
                if connected != Some(true) {
                    connected = Some(true);
                    send_update_to_browser(Status::Connected)?;
                }

                // Hold the connection for a short time and then check if it's still valid
                thread::sleep(Duration::from_secs(2));

                // Check if the socket is still open by calling peer_addr()
                if stream.peer_addr().is_err() && connected != Some(false) {
                    connected = Some(false);
                    send_update_to_browser(Status::Disconnected)?;
                }

                // Drop stream so next loop can reconnect if needed
                drop(stream);
            }
            Err(_) => {
                if connected != Some(false) {
                    connected = Some(false);
                    send_update_to_browser(Status::Disconnected)?;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    }

    fn send_update_to_browser(status: Status) -> io::Result<()> {
        let status_str = match status {
            Status::Connected => "connected",
            Status::Disconnected => "disconnected",
        };

        let message = json!({ "type": status_str }).to_string();
        write_browser_message(&message)?;
        Ok(())
    }

