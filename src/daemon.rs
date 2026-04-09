use chrono::TimeZone;
use crate::config;
use crate::state;
use crate::state::*;
use chrono::Datelike;
use serde_json::Value;
use shire_blocker::*;
use std::collections::HashMap;
use std::fs;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io;

pub fn start_daemon(config_path: Option<String>) {
    let config = config::parse_config(config_path).unwrap();

    let app_state: Arc<Mutex<ApplicationState>> = initialize_application_state(config.clone());

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    let bridge_stream: Arc<Mutex<Option<UnixStream>>> = Arc::new(Mutex::new(None));
    let bridge_app_state = Arc::clone(&app_state);
    let bridge_stream_for_thread = Arc::clone(&bridge_stream);

    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let mut stream_copy = stream.try_clone().expect("Failed to clone stream");

                    // TODO: properly error handle this?
                    let _ = send_state_to_bridge(&mut stream_copy, &bridge_app_state.lock().unwrap());
                    
                    *bridge_stream_for_thread.lock().unwrap() = Some(stream);
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    let bridge_stream_schedule_clone = Arc::clone(&bridge_stream);
    let schedule_app_state = Arc::clone(&app_state);
    thread::spawn(move || {
        let current_day: OrderableWeekday = state::OrderableWeekday(chrono::Local::now().weekday());
        let current_time = chrono::Local::now().time();

        // Find the next event (greater than now)
        // If no event is found for this week, wrap around to the first event next week
        let mut next_event = None;
        let schedule = schedule_app_state.lock().unwrap().schedule.clone();
        
        // First, look for events later this week
        for ev in &schedule {
            if ev.day > current_day || (ev.day == current_day && ev.time > current_time) {
                next_event = Some(ev.clone());
                break;
            }
        }
        
        // If no event found this week, use the first event of next week
        if next_event.is_none() && !schedule.is_empty() {
            next_event = Some(schedule[0].clone());
        }

        loop {
            // Get the time until next event
            if let Some(event) = &next_event {
                let now = chrono::Local::now();
                let current_weekday = now.weekday();
                let current_time_now = now.time();
                
                let target_date = if event.day.0.num_days_from_monday() > current_weekday.num_days_from_monday() || (event.day.0 == current_weekday && event.time > current_time_now) {
                    // Event is later this week
                    let days_until = event.day.0.num_days_from_monday() as i64 - current_weekday.num_days_from_monday() as i64;
                    now.date_naive() + chrono::Duration::days(days_until)
                } else {
                    // Event is next week (including when we wrapped around to first event)
                    let days_until = 7 - current_weekday.num_days_from_monday() as i64 + event.day.0.num_days_from_monday() as i64;
                    now.date_naive() + chrono::Duration::days(days_until)
                };

                let target_datetime = target_date.and_time(event.time);
                let target_local = chrono::Local.from_local_datetime(&target_datetime).unwrap();
                
                if let Ok(duration) = (target_local - now).to_std() {
                    // Sleep until next event
                    println!("Sleeping until {:?}", duration);
                    std::thread::sleep(duration);
                    
                    // Execute the event (I believe that schedules should be blockedwithlock)
                    let mut app_state_guard = schedule_app_state.lock().unwrap();
                    match event.action {
                        state::ScheduleAction::StartBlock => {
                            update_block(&mut app_state_guard, &event.block, BlockState::Blocked);
                        }
                        state::ScheduleAction::EndBlock => {
                            update_block(&mut app_state_guard, &event.block, BlockState::Unblocked);
                        }
                    }

                    // Send updated state to bridge
                    if let Ok(mut bridge_guard) = bridge_stream_schedule_clone.lock() {
                        if let Some(ref mut stream) = *bridge_guard {
                            if let Err(e) = send_state_to_bridge(stream, &app_state_guard) {
                                eprintln!("Failed to send scheduled state update to bridge: {e}");
                            }
                        }
                    }
                    
                    // Go to the next event
                    let current_index = schedule.iter().position(|e| e == event).unwrap_or(0);
                    let next_index = (current_index + 1) % schedule.len();
                    next_event = if next_index < schedule.len() {
                        Some(schedule[next_index].clone())
                    } else {
                        schedule.first().cloned()
                    };
                    drop(app_state_guard);
                } else {
                    // If duration calculation fails, wait a minute and try again
                    std::thread::sleep(std::time::Duration::from_secs(60));
                }
            } else {
                // No events scheduled, break loop and finish thread.
                break;
            }
        }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let cli_app_state = Arc::clone(&app_state);
                let bridge_stream_clone = Arc::clone(&bridge_stream);

                thread::spawn(move || {
                    handle_cli_request(&mut stream, cli_app_state, bridge_stream_clone);
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn handle_cli_request(
    cli_stream: &mut UnixStream,
    app_state: Arc<Mutex<ApplicationState>>,
    bridge_stream: Arc<Mutex<Option<UnixStream>>>,
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

        // TODO: Update this block to handle locks?
        Some("start_block") => {
            if let Some(block_name) = v["name"].as_str() {
                update_block(&mut app_state_guard, block_name, BlockState::Blocked);

                // Send new state to bridge
                if let Ok(mut bridge_guard) = bridge_stream.lock() {
                    if let Some(ref mut stream) = *bridge_guard {
                        if let Err(e) = send_state_to_bridge(stream, &app_state_guard) {
                            eprintln!("Failed to send state to bridge: {e}");
                        }
                    }
                }

                let message = serde_json::json!({ "status": "started", "block": block_name })
                    .to_string()
                    .into_bytes();

                if let Err(e) = send_length_prefixed_message(cli_stream, &message) {
                    eprintln!("Failed to send CLI ack: {e}");
                }
            }
        }

        // TODO: Update this block to error and respond to CLI if a block is locked
        Some("stop_block") => {
            if let Some(block_name) = v["name"].as_str() {
                update_block(&mut app_state_guard, block_name, BlockState::Unblocked);

                // Send new state to bridge
                if let Ok(mut bridge_guard) = bridge_stream.lock() {
                    if let Some(ref mut stream) = *bridge_guard {
                        if let Err(e) = send_state_to_bridge(stream, &app_state_guard) {
                            eprintln!("Failed to send state to bridge: {e}");
                        }
                    }
                }

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

pub fn send_state_to_bridge(
    stream: &mut UnixStream,
    app_state: &ApplicationState,
) -> io::Result<()> {
    // Serialize the state
    let string_map: HashMap<String, &Block> = app_state
        .blocks
        .iter()
        .map(|(name, block)| (name.clone(), block))
        .collect();

    let message = serde_json::json!({
        "type": "state_update",
        "blocks": string_map
    })
    .to_string()
    .into_bytes();

    send_length_prefixed_message(stream, &message)
}
