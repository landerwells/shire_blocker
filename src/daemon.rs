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
use std::sync::{Arc, Mutex};
use std::thread;
use std::io;

// TODO: Move these to bridge
const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

pub fn start_daemon(config_path: Option<String>) {
    let config = config::parse_config(config_path).unwrap();

    let app_state: Arc<Mutex<ApplicationState>> = initialize_application_state(config.clone());

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    // The question is, should I try sending messages on this listener, or wait 
    // and send the messages over the stream
    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    let bridge_stream: Arc<Mutex<Option<UnixStream>>> = Arc::new(Mutex::new(None));
    let bridge_app_state = Arc::clone(&app_state);

    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            // So here I get a stream, and I should send it the initial state, 
            // and then send the stream to the 
            match stream {
                Ok(mut stream) => {
                    // TODO: Connect a single stream and wrap in an Arc Mutex
                    send_state_to_bridge(&mut stream, &bridge_app_state.lock().unwrap());
                    

                    // TODO: Update the bridge stream to hold this new one
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    // let schedule_app_state = Arc::clone(&app_state);
    // thread::spawn(move || {
    //     let current_day: OrderableWeekday = state::OrderableWeekday(chrono::Local::now().weekday());
    //     let current_time = chrono::Local::now().time();
    //
    //     // Iterate through schedule until the next element
    //     // Find the next event (greater than now)
    //     let mut next_event = None;
    //     for ev in &schedule_app_state.lock().unwrap().schedule {
    //         if ev.day > current_day || (ev.day == current_day && ev.time > current_time) {
    //             next_event = Some(ev.clone());
    //             break;
    //         }
    //     }
    //     // loop {
    //     // }
    // });

    // let bridge_cli_stream = Arc::new(Mutex::new(bridge_cli_stream));

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let cli_app_state = Arc::clone(&app_state);
                // let bridge_stream_clone = Arc::clone(&bridge_cli_stream);

                thread::spawn(move || {
                    handle_cli_request(&mut stream, cli_app_state);
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn handle_cli_request(
    cli_stream: &mut UnixStream,
    app_state: Arc<Mutex<ApplicationState>>,
    bridge_stream: Arc<Mutex<Option<UnixStream>>>
) {
    // Read request
    let response = match recv_length_prefixed_message(cli_stream) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to read CLI request: {e}");
            return;
        }
    };

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

            if let Err(e) = send_length_prefixed_message(cli_stream, &message) {
                eprintln!("Failed to send list_blocks response: {e}");
            }
        }

        Some("start_block") => {
            if let Some(block_name) = v["name"].as_str() {
                update_block(&mut app_state_guard, block_name, BlockState::Blocked);

                // TODO; Unlock bridge stream here and send the message
                // Send new state to bridge
                // if let Err(e) = send_state_to_bridge(&bridge_stream, &app_state_guard) {
                //     eprintln!("Failed to send state to bridge: {e}");
                // }

                let message = serde_json::json!({ "status": "started", "block": block_name })
                    .to_string()
                    .into_bytes();

                if let Err(e) = send_length_prefixed_message(cli_stream, &message) {
                    eprintln!("Failed to send CLI ack: {e}");
                }
            }
        }

        Some("stop_block") => {
            if let Some(block_name) = v["name"].as_str() {
                update_block(&mut app_state_guard, block_name, BlockState::Unblocked);

                // TODO; Unlock bridge stream here and send the message
                // Send new state to bridge
                // Send new state to bridge
                // if let Err(e) = send_state_to_bridge(&bridge_stream, &app_state_guard) {
                //     eprintln!("Failed to send state to bridge: {e}");
                // }

                let message = serde_json::json!({ "status": "stopped", "block": block_name })
                    .to_string()
                    .into_bytes();

                if let Err(e) = send_length_prefixed_message(cli_stream, &message) {
                    eprintln!("Failed to send CLI ack: {e}");
                }
            }
        }

        _ => eprintln!("Unknown action in CLI request."),
    }
}

/// Send the current application state to the bridge
pub fn send_state_to_bridge(
    stream: &mut UnixStream,
    app_state: &ApplicationState,
) -> io::Result<()> {
    // Serialize the state
    let string_map: HashMap<String, BlockState> = app_state
        .blocks
        .iter()
        .map(|(name, block)| (name.clone(), block.block_state))
        .collect();

    let message = serde_json::json!({
        "type": "state_update",
        "blocks": string_map
    })
    .to_string()
    .into_bytes();

    // Send over the bridge
    send_length_prefixed_message(stream, &message)
}
