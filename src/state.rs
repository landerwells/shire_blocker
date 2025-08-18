use crate::config::Config;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ApplicationState {
    pub blocks: HashMap<String, Block>,
    pub schedule: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Block {
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
    pub block_state: BlockState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum BlockState {
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
    Sunday,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    block: String,
    day: Days,
    hour: i32,
    minute: i32,
    action: ScheduleAction,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ScheduleAction {
    StartBlock,
    EndBlock,
}

pub fn initialize_application_state(config: Config) -> Arc<Mutex<ApplicationState>> {
    let application_state = Arc::new(Mutex::new(ApplicationState {
        blocks: HashMap::new(),
        schedule: Vec::new(),
    }));

    // Block initialization

    config.blocks.iter().for_each(|block| {
        let state = if matches!(block.active_by_default, Some(true)) {
            BlockState::Blocked
        } else {
            BlockState::Unblocked
        };

        // let mut map = block_states.lock().unwrap();
        // map.insert(block.clone(), state);
        application_state.lock().unwrap().blocks.insert(
            block.name.clone(),
            Block {
                whitelist: block.whitelist.clone(),
                blacklist: block.blacklist.clone(),
                block_state: state,
            },
        );
    });

    // Schedule initialization
    // TODO: This is not working at all.
    let schedules = config.schedule.clone();

    let mut weekly_schedule: Vec<Event> = Vec::new();
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
                    weekly_schedule.push(create_event(
                        day_enum.clone(),
                        hour,
                        minute,
                        block_name.clone(),
                        ScheduleAction::StartBlock,
                    ));
                }
                Err(e) => {
                    eprintln!(
                        "Skipping invalid start time '{}' for {}: {}",
                        schedule.start, day, e
                    );
                    continue;
                }
            }

            // Add the end time
            match parse_time(&schedule.end) {
                Ok((hour, minute)) => {
                    weekly_schedule.push(create_event(
                        day_enum,
                        hour,
                        minute,
                        block_name,
                        ScheduleAction::EndBlock,
                    ));
                }
                Err(e) => {
                    eprintln!(
                        "Skipping invalid end time '{}' for {}: {}",
                        schedule.end, day, e
                    );
                }
            }
        }
    }

    weekly_schedule.sort();
    application_state.lock().unwrap().schedule = weekly_schedule;
    application_state
}

// The validation can be removed from this function in favor of validating in
// config.rs, and instead this can just be to parse out the hour and minute.
// Should be moved to state.rs
fn parse_time(time_str: &str) -> Result<(i32, i32), String> {
    if time_str.len() < 5 || !time_str.contains(':') {
        return Err(format!("Invalid time format: {time_str}"));
    }

    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid time format: {time_str}"));
    }

    let hour = parts[0]
        .parse::<i32>()
        .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
    let minute = parts[1]
        .parse::<i32>()
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
    DAY_MAP
        .iter()
        .find(|(d, _)| *d == day_str)
        .map(|(_, day)| day.clone())
        .ok_or_else(|| format!("Invalid day: {day_str}"))
}

pub fn update_block(application_state: &mut ApplicationState, block_name: &str, new_state: BlockState) {
    if let Some(block) = application_state.blocks.get_mut(block_name) {
        block.block_state = new_state;
    } else {
        eprintln!("Block '{}' not found in application state", block_name);
    }
}

fn create_event(day: Days, hour: i32, minute: i32, block: String, action: ScheduleAction) -> Event {
    Event {
        day,
        hour,
        minute,
        block,
        action,
    }
}
