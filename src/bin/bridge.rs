use serde_json::{json, Value};
use shire_blocker::{recv_length_prefixed_message, send_length_prefixed_message};
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// These should probably go in lib?
#[derive(Clone, Debug)]
struct ApplicationState {
    active_blocks: Vec<BlockConfig>,
    daemon_connected: bool,
}

#[derive(Clone, Debug)]
struct BlockConfig {
    name: String,
    blacklist: Option<Vec<String>>,
    whitelist: Option<Vec<String>>,
    locked_until: Option<u64>,
}

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

// Bridge should be the unix listener now, and the daemon will fail to start
// if the extension is not installed.
fn connect_to_daemon() -> io::Result<UnixStream> {
    UnixStream::connect("/tmp/shire_bridge.sock")
}

fn request_state_from_daemon(stream: &mut UnixStream) -> io::Result<ApplicationState> {
    let request = json!({
        "action": "get_state"
    });
    
    send_length_prefixed_message(stream, request.to_string().as_bytes())?;
    
    match recv_length_prefixed_message(stream) {
        Ok(response) => {
            let response_str = String::from_utf8_lossy(&response);
            let v: Value = serde_json::from_str(&response_str).unwrap_or_else(|_| {
                eprintln!("Invalid JSON response from daemon");
                json!({})
            });

            let mut active_blocks = Vec::new();
            if let Some(blocks) = v["active_blocks"].as_array() {
                for block in blocks {
                    let mut blacklist = None;
                    let mut whitelist = None;
                    
                    if let Some(bl) = block["blacklist"].as_array() {
                        blacklist = Some(bl.iter().filter_map(|s| s.as_str().map(String::from)).collect());
                    }
                    
                    if let Some(wl) = block["whitelist"].as_array() {
                        whitelist = Some(wl.iter().filter_map(|s| s.as_str().map(String::from)).collect());
                    }

                    active_blocks.push(BlockConfig {
                        name: block["name"].as_str().unwrap_or("unknown").to_string(),
                        blacklist,
                        whitelist,
                        locked_until: block["locked_until"].as_u64(),
                    });
                }
            }

            Ok(ApplicationState {
                active_blocks,
                daemon_connected: true,
            })
        }
        Err(e) => {
            eprintln!("Failed to get state from daemon: {e}");
            Ok(ApplicationState {
                active_blocks: Vec::new(),
                daemon_connected: false,
            })
        }
    }
}

fn send_state_to_browser(state: &ApplicationState) -> io::Result<()> {
    let state_message = json!({
        "type": "state_update",
        "state": {
            "active_blocks": state.active_blocks.iter().map(|block| {
                json!({
                    "name": block.name,
                    "blacklist": block.blacklist,
                    "whitelist": block.whitelist,
                    "locked_until": block.locked_until
                })
            }).collect::<Vec<_>>(),
            "daemon_connected": state.daemon_connected
        }
    });

    write_browser_message(&state_message.to_string())
}

fn main() -> io::Result<()> {
    let current_state = Arc::new(Mutex::new(ApplicationState {
        active_blocks: Vec::new(),
        daemon_connected: false,
    }));

    let state_for_poller = Arc::clone(&current_state);
    thread::spawn(move || {
        loop {
            match connect_to_daemon() {
                Ok(mut stream) => {
                    match request_state_from_daemon(&mut stream) {
                        Ok(new_state) => {
                            let mut state_lock = state_for_poller.lock().unwrap();
                            let state_changed = state_lock.active_blocks.len() != new_state.active_blocks.len() 
                                || state_lock.daemon_connected != new_state.daemon_connected;
                            
                            *state_lock = new_state;
                            
                            if state_changed {
                                if let Err(e) = send_state_to_browser(&state_lock) {
                                    eprintln!("Failed to send state to browser: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get state from daemon: {e}");
                            let mut state_lock = state_for_poller.lock().unwrap();
                            if state_lock.daemon_connected {
                                state_lock.daemon_connected = false;
                                if let Err(e) = send_state_to_browser(&state_lock) {
                                    eprintln!("Failed to send disconnection state to browser: {e}");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to daemon: {e}");
                    let mut state_lock = state_for_poller.lock().unwrap();
                    if state_lock.daemon_connected {
                        state_lock.daemon_connected = false;
                        if let Err(e) = send_state_to_browser(&state_lock) {
                            eprintln!("Failed to send disconnection state to browser: {e}");
                        }
                    }
                }
            }
            
            thread::sleep(Duration::from_secs(1));
        }
    });

    if let Ok(mut stream) = connect_to_daemon() {
        if let Ok(initial_state) = request_state_from_daemon(&mut stream) {
            let mut state_lock = current_state.lock().unwrap();
            *state_lock = initial_state;
            send_state_to_browser(&state_lock)?;
        }
    }

    loop {
        match read_browser_message() {
            Ok(message) => {
                let message_str = String::from_utf8_lossy(&message);
                eprintln!("Received from browser: {}", message_str);
                
                let response = json!({
                    "type": "pong",
                    "message": "Bridge is alive"
                });
                
                if let Err(e) = write_browser_message(&response.to_string()) {
                    eprintln!("Failed to send response to browser: {e}");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read browser message: {e}");
                break;
            }
        }
    }

    Ok(())
}
