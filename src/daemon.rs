use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use shire_blocker::recv_length_prefixed_message;
use shire_blocker::send_length_prefixed_message;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use chrono::{Local, Timelike, Datelike, Weekday};
use std::{thread, time::Duration};
use std::thread;
use crate::config::*;


const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum BlockState {
    Unblocked,
    Blocked,
    BlockedWithLock,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
enum Days {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}

const DAY_MAP: &[(&str, Days)] = &[
    ("Mon", Days::Monday),
    ("Tue", Days::Tuesday),
    ("Wed", Days::Wednesday),
    ("Thu", Days::Thursday),
    ("Fri", Days::Friday),
    ("Sat", Days::Saturday),
    ("Sun", Days::Sunday),
];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Event {
    day: Days,
    hour: i32,
    minute: i32,
    block: String,
    action: ScheduleAction
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ScheduleAction {
    StartBlock,
    EndBlock
}

// Parse time could eventually be moved into the configuration parsing? 
// I am thinking that invalid parsing will not reload the current configuration
// and the application will just persist if the config parsing fails.
fn parse_time(time_str: &str) -> Result<(i32, i32), String> {
    if time_str.len() < 5 || !time_str.contains(':') {
        return Err(format!("Invalid time format: {time_str}"));
    }
    
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid time format: {time_str}"));
    }
    
    let hour = parts[0].parse::<i32>()
        .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
    let minute = parts[1].parse::<i32>()
        .map_err(|_| format!("Invalid minute: {}", parts[1]))?;
    
    if !(0..=23).contains(&hour) {
        return Err(format!("Hour out of range (0-23): {hour}"));
    }
    if !(0..=59).contains(&minute) {
        return Err(format!("Minute out of range (0-59): {minute}"));
    }
    
    Ok((hour, minute))
}

fn parse_day(day_str: &str) -> Result<Days, String> {
    DAY_MAP.iter()
        .find(|(d, _)| *d == day_str)
        .map(|(_, day)| day.clone())
        .ok_or_else(|| format!("Invalid day: {day_str}"))
}

fn create_event(day: Days, hour: i32, minute: i32, block: String, action: ScheduleAction) -> Event {
    Event { day, hour, minute, block, action }
}

fn weekday_to_days(weekday: Weekday) -> Days {
    match weekday {
        Weekday::Mon => Days::Monday,
        Weekday::Tue => Days::Tuesday,
        Weekday::Wed => Days::Wednesday,
        Weekday::Thu => Days::Thursday,
        Weekday::Fri => Days::Friday,
        Weekday::Sat => Days::Saturday,
        Weekday::Sun => Days::Sunday,
    }
}

fn get_current_day_time() -> (Days, i32, i32) {
    let now = Local::now();
    let day = weekday_to_days(now.weekday());
    let hour = now.hour() as i32;
    let minute = now.minute() as i32;
    (day, hour, minute)
}

pub fn start_daemon() {
    let config = parse_config().unwrap();
    let schedules = config.schedule;

    let mut weekly_schedule: Vec<Event> = Vec::new();

    // Parse configuration schedules into events for the week

    for schedule in schedules {
        for day in schedule.days {
            let day_enum = match parse_day(&day) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Skipping invalid day '{}': {}", day, e);
                    continue;
                }
            };
            let block_name = schedule.block.clone();

            // Add the start time
            match parse_time(&schedule.start) {
                Ok((hour, minute)) => {
                    weekly_schedule.push(create_event(day_enum.clone(), hour, minute, block_name.clone(), ScheduleAction::StartBlock));
                }
                Err(e) => {
                    eprintln!("Skipping invalid start time '{}' for {}: {}", schedule.start, day, e);
                    continue;
                }
            }

            // Add the end time
            match parse_time(&schedule.end) {
                Ok((hour, minute)) => {
                    weekly_schedule.push(create_event(day_enum, hour, minute, block_name, ScheduleAction::EndBlock));
                }
                Err(e) => {
                    eprintln!("Skipping invalid end time '{}' for {}: {}", schedule.end, day, e);
                }
            }
        }
    }

    // Sort the schedule chronologically
    weekly_schedule.sort();

    let block_states = Arc::new(Mutex::new(HashMap::<Block, BlockState>::new()));

    config.blocks.iter().for_each(|block| {
        let state = if matches!(block.active_by_default, Some(true)) {
            BlockState::Blocked
        } else {
            BlockState::Unblocked
        };

        let mut map = block_states.lock().unwrap();
        map.insert(block.clone(), state);
    });

    // Initialize block states based on configuration

    let _ = fs::remove_file(BRIDGE_SOCKET_PATH);
    let _ = fs::remove_file(CLI_SOCKET_PATH);

    let bridge_listener = UnixListener::bind(BRIDGE_SOCKET_PATH).unwrap();
    let cli_listener = UnixListener::bind(CLI_SOCKET_PATH).unwrap();

    let blocks = Arc::clone(&block_states);
    thread::spawn(move || {
        for stream in bridge_listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let _ = handle_bridge_request(&mut stream, &blocks.lock().unwrap().clone());
                }
                Err(e) => eprintln!("Bridge connection failed: {e}"),
            }
        }
    });

    let schedule_blocks = Arc::clone(&block_states);
    thread::spawn(move || {
        loop {
            let (current_day, current_hour, current_minute) = get_current_day_time();
            
            // Find events that should trigger now
            for event in &weekly_schedule {
                if event.day == current_day && event.hour == current_hour && event.minute == current_minute {
                    println!("Triggering event: {:?}", event);
                    
                    let mut map = schedule_blocks.lock().unwrap();
                    
                    // Find the block to update
                    if let Some((block, state)) = map.iter_mut().find(|(b, _)| b.name == event.block) {
                        match event.action {
                            ScheduleAction::StartBlock => {
                                *state = BlockState::BlockedWithLock;
                                println!("Started block: {}", block.name);
                            }
                            ScheduleAction::EndBlock => {
                                *state = BlockState::Unblocked;
                                println!("Ended block: {}", block.name);
                            }
                        }
                    } else {
                        eprintln!("Block '{}' not found for scheduled event", event.block);
                    }
                }
            }
            
            // Sleep for 1 minute before checking again
            thread::sleep(Duration::from_secs(60));
        }
    });

    for stream in cli_listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let blocks = Arc::clone(&block_states);

                thread::spawn(move || {
                    handle_cli_request(&mut stream, blocks.clone());
                });
            }
            Err(e) => eprintln!("CLI connection failed: {e}"),
        }
    }
}

fn get_blacklist(
    blocks: &HashMap<Block, BlockState>,
) -> HashSet<String> {
    blocks
        .iter()
        .filter_map(|(block, state)| {
            if *state == BlockState::Blocked || *state == BlockState::BlockedWithLock {
                block.blacklist.clone()
            } else {
                None
            }
        })
    .flatten()
        .collect()
}

fn get_whitelist(
    blocks: &HashMap<Block, BlockState>,
) -> HashSet<String> {
    blocks
        .iter()
        .filter_map(|(block, state)| {
            if *state == BlockState::Blocked || *state == BlockState::BlockedWithLock {
                block.whitelist.clone()
            } else {
                None
            }
        })
    .flatten()
        .collect()
}

fn handle_cli_request(stream: &mut UnixStream, blocks: Arc<Mutex<HashMap<Block, BlockState>>>) {
    // Need to have better error handling here
    let response = recv_length_prefixed_message(stream).unwrap();
    let response_str = String::from_utf8_lossy(&response);

    let v: Value = serde_json::from_str(response_str.trim()).unwrap_or_else(|_| {
        eprintln!("Invalid JSON from CLI.");
        serde_json::json!({})
    });

    let mut map = blocks.lock().unwrap();

    match v["action"].as_str() {
        Some("list_blocks") => {

            let string_map: HashMap<String, BlockState> = map
                .iter()
                .map(|(block, state)| (block.name.clone(), *state))
                .collect();

            let message = serde_json::json!({ "blocks": string_map }).to_string().into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        // I could even make a function purely for changing the state of the blocks
        Some("start_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();
            if let Some(state) = map.iter_mut().find_map(|(b, state)| {
                if b.name == block_name { Some(state) } else { None }
            }) {
                *state = BlockState::Blocked;
            } else {
                eprintln!("Block '{block_name}' not found.");
            }

            let message = serde_json::json!({ "status": "started", "block": block_name }).to_string().into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        Some("stop_block") => {
            let block_name = v["name"].as_str().unwrap().to_string();

            // Use iter_mut and find_map just like in start_block to avoid cloning keys
            if let Some(state) = map.iter_mut().find_map(|(b, state)| {
                if b.name == block_name { Some(state) } else { None }
            }) {
                *state = BlockState::Unblocked;
            } else {
                eprintln!("Block '{block_name}' not found.");
            }

            let message = serde_json::json!({ "status": "stopped", "block": block_name }).to_string().into_bytes();
            send_length_prefixed_message(stream, &message).unwrap();
        }
        _ => eprintln!("Unknown action in CLI request."),
    }
}

fn remove_http_www(mut url_string: String) -> String {
    if url_string.starts_with("https://") {
        url_string = url_string.strip_prefix("https://").unwrap().to_string();
    }

    if url_string.starts_with("www.") {
        url_string = url_string.strip_prefix("www.").unwrap().to_string();
    }

    url_string
}

fn is_blacklisted(blacklist: &HashSet<String>, url: &str) -> bool {
    let url = remove_http_www(url.to_string());
    blacklist.iter().any(|entry| url.starts_with(entry))
}

fn is_whitelisted(whitelist: &HashSet<String>, url: &str) -> bool {
    let url = remove_http_www(url.to_string());

    whitelist.iter().any(|pattern| {
        let prefix = pattern.trim_end_matches('*');
        url.starts_with(prefix)
    })
}

fn handle_bridge_request(
    stream: &mut UnixStream,
    blocks: &HashMap<Block, BlockState>,
) -> io::Result<()> {
    // Get black/white lists
    let blacklist = get_blacklist(blocks);
    let whitelist = get_whitelist(blocks);

    // Receive length-prefixed JSON request
    let raw_request = recv_length_prefixed_message(stream)?;
    let request_str = String::from_utf8_lossy(&raw_request);
    println!("Bridge request: {}", request_str);

    let v: Value = serde_json::from_str(request_str.trim()).unwrap_or_else(|_| {
        eprintln!("Invalid JSON from bridge.");
        json!({})
    });

    let url = v["url"].as_str().unwrap_or("").to_string();
    let url = remove_http_www(url);

    println!("Checking URL: {}", url);

    // Check if allowed or blocked
    let allowed = !is_blacklisted(&blacklist, &url) || is_whitelisted(&whitelist, &url);

    println!(
        "{} URL from bridge: {}",
        if allowed { "Allowed" } else { "Blocked" },
        url
    );

    // Send JSON response with "allowed" key
    let response_json = json!({ "allowed": allowed });
    let response_bytes = response_json.to_string().into_bytes();
    send_length_prefixed_message(stream, &response_bytes)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json() {
        let json_str =
            r#"{"url":"https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"}"#;

        let v: Value = serde_json::from_str(json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {json_str}");
            Value::Null
        });
        assert_eq!(
            v["url"],
            "https://www.google.com/search?client=firefox-b-1-d&q=json+parser+rust"
        );

        // Test with an invalid JSON string
        let invalid_json_str = r#"{"url": "https://www.example.com""#;
        let v_invalid: Value = serde_json::from_str(invalid_json_str).unwrap_or_else(|_| {
            eprintln!("Failed to parse JSON: {invalid_json_str}");
            Value::Null
        });
        assert!(v_invalid.is_null());
    }

    #[test]
    fn test_remove_http_www() {
        let url_with_http = "https://www.example.com".to_string();
        let url_without_http = remove_http_www(url_with_http);
        assert_eq!(url_without_http, "example.com".to_string());

        let url_with_https = "https://example.com".to_string();
        let url_without_https = remove_http_www(url_with_https);
        assert_eq!(url_without_https, "example.com".to_string());

        let url_with_www = "www.example.com".to_string();
        let url_without_www = remove_http_www(url_with_www);
        assert_eq!(url_without_www, "example.com".to_string());
    }
}
