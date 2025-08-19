use crate::config;
use crate::state;
use crate::state::*;
use chrono::Datelike;
use serde_json::Value;
use serde_json::json;
use shire_blocker::recv_length_prefixed_message;
use shire_blocker::send_length_prefixed_message;
use std::collections::HashMap;
use std::fs;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

pub fn start_daemon(config_path: Option<String>) {
    let config = config::parse_config(config_path).unwrap();

    let app_state: Arc<Mutex<ApplicationState>> = initialize_application_state(config.clone());

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    // Channel for notifying bridge thread of state changes
    let (state_tx, state_rx) = mpsc::channel();

    // TODO: link to where someone can download the browser extension.
    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let (mut bridge_stream, _addr) = bridge_listener.accept().unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    // Bridge thread - waits for state change notifications and sends state to bridge
    let bridge_app_state = Arc::clone(&app_state);
    thread::spawn(move || {
        // Send initial state
        if let Err(e) = send_state_to_bridge(&mut bridge_stream, &bridge_app_state) {
            eprintln!("Failed to send initial state to bridge: {e}");
        }
        
        // Wait for state change notifications
        while state_rx.recv().is_ok() {
            if let Err(e) = send_state_to_bridge(&mut bridge_stream, &bridge_app_state) {
                eprintln!("Failed to send state update to bridge: {e}");
                break;
            }
        }
    });

    let schedule_app_state = Arc::clone(&app_state);
    thread::spawn(move || {
        // Get current day and time
        let current_day: OrderableWeekday = state::OrderableWeekday(chrono::Local::now().weekday());
        let current_time = chrono::Local::now().time();

        // println!("Current day: {:?}, current time {:?}", current_day, current_time);

        // Iterate through schedule until the next element
        // Find the next event (greater than now)
        let mut next_event = None;
        for ev in &schedule_app_state.lock().unwrap().schedule {
            if ev.day > current_day || (ev.day == current_day && ev.time > current_time) {
                next_event = Some(ev.clone());
                break;
            }
        }

        // loop {
        // }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let cli_app_state = Arc::clone(&app_state);
                let cli_state_tx = state_tx.clone();

                thread::spawn(move || {
                    handle_cli_request(&mut stream, cli_app_state, cli_state_tx);
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn send_state_to_bridge(stream: &mut UnixStream, app_state: &Arc<Mutex<ApplicationState>>) -> Result<(), Box<dyn std::error::Error>> {
    let app_state_guard = app_state.lock().unwrap();
    
    let state_message = json!({
        "type": "state_update",
        "state": {
            "blocks": app_state_guard.blocks
        }
    });
    
    let message_bytes = state_message.to_string().into_bytes();
    send_length_prefixed_message(stream, &message_bytes)?;
    
    Ok(())
}

fn handle_cli_request(stream: &mut UnixStream, app_state: Arc<Mutex<ApplicationState>>, state_tx: mpsc::Sender<()>) {
    // Need to have better error handling here
    let response = recv_length_prefixed_message(stream).unwrap();
    let response_str = String::from_utf8_lossy(&response);

    let v: Value = serde_json::from_str(response_str.trim()).unwrap_or_else(|_| {
        eprintln!("Invalid JSON from CLI.");
        serde_json::json!({})
    });

    let mut app_state_guard = app_state.lock().unwrap();

    match v["action"].as_str() {
        Some("list_blocks") => {
            let string_map: HashMap<String, BlockState> = app_state_guard
                .blocks
                .iter()
                .map(|(name, block)| (name.clone(), block.block_state))
                .collect();

            let message = serde_json::json!({ "blocks": string_map })
                .to_string()
                .into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        // I could even make a function purely for changing the state of the blocks
        Some("start_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();
            update_block(&mut app_state_guard, &block_name, BlockState::Blocked, state_tx);

            let message = serde_json::json!({ "status": "started", "block": block_name })
                .to_string()
                .into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        Some("stop_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();
            update_block(&mut app_state_guard, &block_name, BlockState::Unblocked, state_tx);

            let message = serde_json::json!({ "status": "stopped", "block": block_name })
                .to_string()
                .into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        _ => eprintln!("Unknown action in CLI request."),
    }
}

