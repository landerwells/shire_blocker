use chrono::TimeZone;
use crate::config;
use crate::state;
use crate::state::*;
use chrono::Datelike;
use serde_json::Value;
use shire_blocker::*;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex as AsyncMutex;

// Holds the full stream (not a split write-half) so that dropping one half
// never triggers SHUT_RD and disconnects the bridge.
type BridgeConn = Arc<AsyncMutex<Option<UnixStream>>>;

pub async fn start_daemon(config_path: Option<String>) {
    let config = config::parse_config(config_path).unwrap();
    let app_state: Arc<Mutex<ApplicationState>> = initialize_application_state(config.clone());

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    let bridge_conn: BridgeConn = Arc::new(AsyncMutex::new(None));

    // Bridge listener task
    let bridge_conn_for_accept = Arc::clone(&bridge_conn);
    let bridge_app_state = Arc::clone(&app_state);
    tokio::spawn(async move {
        loop {
            match bridge_listener.accept().await {
                Ok((mut stream, _)) => {
                    let state_bytes = serialize_state(&bridge_app_state.lock().unwrap());
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        send_prefixed_bytes(&mut stream, &state_bytes),
                    )
                    .await
                    {
                        Ok(Ok(())) => {
                            let mut guard = bridge_conn_for_accept.lock().await;
                            if guard.is_some() {
                                eprintln!("Bridge reconnected, replacing existing connection");
                            }
                            *guard = Some(stream);
                        }
                        Ok(Err(e)) => eprintln!("Bridge initial state send failed: {e}"),
                        Err(_) => eprintln!("Bridge initial state send timed out"),
                    }
                }
                Err(e) => eprintln!("Bridge accept failed: {e}"),
            }
        }
    });

    // Schedule task
    let bridge_conn_for_schedule = Arc::clone(&bridge_conn);
    let schedule_app_state = Arc::clone(&app_state);
    tokio::spawn(async move {
        let current_day = state::OrderableWeekday(chrono::Local::now().weekday());
        let current_time = chrono::Local::now().time();

        let schedule = schedule_app_state.lock().unwrap().schedule.clone();

        let mut next_event = schedule
            .iter()
            .find(|ev| ev.day > current_day || (ev.day == current_day && ev.time > current_time))
            .or_else(|| schedule.first())
            .cloned();

        loop {
            let Some(event) = &next_event else { break };

            let now = chrono::Local::now();
            let current_weekday = now.weekday();
            let current_time_now = now.time();

            let target_date = if event.day.0.num_days_from_monday() > current_weekday.num_days_from_monday()
                || (event.day.0 == current_weekday && event.time > current_time_now)
            {
                let days_until = event.day.0.num_days_from_monday() as i64
                    - current_weekday.num_days_from_monday() as i64;
                now.date_naive() + chrono::Duration::days(days_until)
            } else {
                let days_until = 7 - current_weekday.num_days_from_monday() as i64
                    + event.day.0.num_days_from_monday() as i64;
                now.date_naive() + chrono::Duration::days(days_until)
            };

            let target_datetime = target_date.and_time(event.time);
            let target_local = chrono::Local.from_local_datetime(&target_datetime).unwrap();

            match (target_local - now).to_std() {
                Ok(duration) => {
                    println!("Sleeping until {:?}", duration);
                    tokio::time::sleep(duration).await;

                    let state_bytes = {
                        let mut guard = schedule_app_state.lock().unwrap();
                        match event.action {
                            state::ScheduleAction::StartBlock => {
                                update_block(&mut guard, &event.block, BlockState::Blocked);
                            }
                            state::ScheduleAction::EndBlock => {
                                update_block(&mut guard, &event.block, BlockState::Unblocked);
                            }
                        }
                        serialize_state(&guard)
                    };

                    send_to_bridge(&bridge_conn_for_schedule, &state_bytes).await;

                    let current_index = schedule.iter().position(|e| e == event).unwrap_or(0);
                    let next_index = (current_index + 1) % schedule.len();
                    next_event = Some(schedule[next_index].clone());
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
            }
        }
    });

    // CLI listener loop
    loop {
        match cli_listener.accept().await {
            Ok((mut stream, _)) => {
                let cli_app_state = Arc::clone(&app_state);
                let bridge_conn_clone = Arc::clone(&bridge_conn);
                tokio::spawn(async move {
                    handle_cli_request(&mut stream, cli_app_state, bridge_conn_clone).await;
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

async fn handle_cli_request(
    cli_stream: &mut UnixStream,
    app_state: Arc<Mutex<ApplicationState>>,
    bridge_conn: BridgeConn,
) {
    let response = match recv_length_prefixed_message_async(cli_stream).await {
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

    match v["action"].as_str() {
        Some("list_blocks") => {
            let message = {
                let guard = app_state.lock().unwrap();
                let string_map: HashMap<String, BlockState> = guard
                    .blocks
                    .iter()
                    .map(|(name, block)| (name.clone(), block.block_state))
                    .collect();
                serde_json::json!({ "blocks": string_map }).to_string().into_bytes()
            };

            if let Err(e) = send_length_prefixed_message_async(cli_stream, &message).await {
                eprintln!("Failed to send list_blocks response: {e}");
            }
        }

        // TODO: Update this block to handle locks?
        Some("start_block") => {
            if let Some(block_name) = v["name"].as_str().map(String::from) {
                let state_bytes = {
                    let mut guard = app_state.lock().unwrap();
                    update_block(&mut guard, &block_name, BlockState::Blocked);
                    serialize_state(&guard)
                };

                send_to_bridge(&bridge_conn, &state_bytes).await;

                let ack = serde_json::json!({ "status": "started", "block": block_name })
                    .to_string()
                    .into_bytes();
                if let Err(e) = send_length_prefixed_message_async(cli_stream, &ack).await {
                    eprintln!("Failed to send CLI ack: {e}");
                }
            }
        }

        // TODO: Update this block to error and respond to CLI if a block is locked
        Some("stop_block") => {
            if let Some(block_name) = v["name"].as_str().map(String::from) {
                let state_bytes = {
                    let mut guard = app_state.lock().unwrap();
                    update_block(&mut guard, &block_name, BlockState::Unblocked);
                    serialize_state(&guard)
                };

                send_to_bridge(&bridge_conn, &state_bytes).await;

                let ack = serde_json::json!({ "status": "stopped", "block": block_name })
                    .to_string()
                    .into_bytes();
                if let Err(e) = send_length_prefixed_message_async(cli_stream, &ack).await {
                    eprintln!("Failed to send CLI ack: {e}");
                }
            }
        }

        _ => eprintln!("Unknown action in CLI request."),
    }
}

async fn send_to_bridge(bridge_conn: &BridgeConn, bytes: &[u8]) {
    let mut guard = bridge_conn.lock().await;
    if let Some(ref mut stream) = *guard {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            send_prefixed_bytes(stream, bytes),
        )
        .await;

        let failed = match result {
            Ok(Ok(())) => false,
            Ok(Err(e)) => {
                eprintln!("Bridge write failed, dropping connection: {e}");
                true
            }
            Err(_) => {
                eprintln!("Bridge write timed out, dropping connection");
                true
            }
        };

        if failed {
            *guard = None;
        }
    }
}

fn serialize_state(app_state: &ApplicationState) -> Vec<u8> {
    let string_map: HashMap<String, &Block> = app_state
        .blocks
        .iter()
        .map(|(name, block)| (name.clone(), block))
        .collect();
    serde_json::json!({
        "type": "state_update",
        "blocks": string_map
    })
    .to_string()
    .into_bytes()
}

async fn send_prefixed_bytes(stream: &mut UnixStream, message: &[u8]) -> io::Result<()> {
    let length = (message.len() as u32).to_be_bytes();
    stream.write_all(&length).await?;
    stream.write_all(message).await?;
    Ok(())
}
