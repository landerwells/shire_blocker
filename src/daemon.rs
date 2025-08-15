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
use chrono::{Local, Timelike, Datelike};
use std::{thread, time::Duration};
use crate::config::*;


const BRIDGE_SOCKET_PATH: &str = "/tmp/shire_bridge.sock";
const CLI_SOCKET_PATH: &str = "/tmp/shire_cli.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum BlockState {
    Unblocked,
    Blocked,
    BlockedWithLock,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
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

// Pretty sure there could be some good error handling cases with this event thing.
// should put a lot of the error handling in the config parsing though.
#[derive(Debug)]
struct Event {
    day: Days,
    hour: i32, // I could constrain this to be 0-23 
    minute: i32, // and this to be 0-59
    action: ScheduleAction
}

#[derive(Debug)]
enum ScheduleAction {
    StartBlock,
    EndBlock
}

pub fn start_daemon() {
    let config = parse_config().unwrap();
    let schedules = config.schedule;

    let mut weekly_schedule: Vec<Event> = Vec::new();

    // Basically, we have the entire schedule for a week. Will need to figure out 
    // how to loop it? Can probably just get the next scheduled event, and have
    // it wrap around when completed. Different schedules on different days should
    // be able to be specified by simply making another schedule element for them.
    

    // Filter them into some data structure to help alleviate sorting. Essentially
    // should have day, time, action, on what block. This should be all of the necessary
    // data for starting and stopping blocks via schedule.
    //
    // From the suggestions of ChatGPT, there was the use of a min_heap which could
    // sort all of the scheduled events and just keep popping, but I don't 
    // necessarily like this idea, because there is no need to remove events from
    // the data structure once they have passed. It 
    //
    // The data structure will contain the days, with a vector of events
    //
    // Eventually if I put locking into the database, there should be no way for
    // the user to unlock their blocks by changing their configuration file.

    // Build the weekly schedule
    for schedule in schedules {
        for day in schedule.days {
            // Add the start time
            let hour = schedule.start[0..2].parse::<i32>().unwrap();
            let minute = schedule.start[3..schedule.start.len()].parse::<i32>().unwrap();

            // parse hours and minutes from time
            let event = Event {
                day: match DAY_MAP.iter().find(|(d, _)| *d == day.as_str()) {
                    Some((_, d)) => d.clone(),
                    None => continue, // Skip invalid days
                },
                hour,
                minute,
                action: ScheduleAction::StartBlock
            };
            weekly_schedule.push(event);

            // Add the end time
            let hour = schedule.end[0..2].parse::<i32>().unwrap();
            let minute = schedule.end[3..schedule.end.len()].parse::<i32>().unwrap();
            let end_event = Event {
                day: match DAY_MAP.iter().find(|(d, _)| *d == day.as_str()) {
                    Some((_, d)) => d.clone(),
                    None => continue, // Skip invalid days
                },
                hour,
                minute,
                action: ScheduleAction::EndBlock
            };
            weekly_schedule.push(end_event);
        }
    }

    // weekly_schedule.sort_by(|a, b| {
    //     a.day.cmp(&b.day)
    //         .then(a.hour.cmp(&b.hour))
    //         .then(a.minute.cmp(&b.minute))
    // });
    
    for event in weekly_schedule {
        println!("Event: {:?}", event);
    }


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

    // Update the blocks in the config based off schedule

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

    // I guess this would be the scheduling or event thread since it would process
    // not just the schedule as well, it would also need to process when locks 
    // in the database are over. Hot reloading the config would further complicate
    // this but I am still not done with that idea.
    thread::spawn(move || {

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
