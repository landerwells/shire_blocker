use crate::config::Config;
use chrono::NaiveTime;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use chrono::Weekday;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderableWeekday(pub Weekday);

impl From<Weekday> for OrderableWeekday {
    fn from(weekday: Weekday) -> Self {
        OrderableWeekday(weekday)
    }
}

impl Ord for OrderableWeekday {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.num_days_from_monday().cmp(&other.0.num_days_from_monday())
    }
}

impl PartialOrd for OrderableWeekday {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    pub block: String,
    pub day: OrderableWeekday,
    pub time: NaiveTime,
    pub action: ScheduleAction,
}

const DAY_MAP: &[(&str, Weekday)] = &[
    ("Mon", Weekday::Mon),
    ("Tue", Weekday::Tue),
    ("Wed", Weekday::Wed),
    ("Thu", Weekday::Thu),
    ("Fri", Weekday::Fri),
    ("Sat", Weekday::Sat),
    ("Sun", Weekday::Sun),
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
            let start_time = parse_time(&schedule.start);
            weekly_schedule.push(create_event(
                day_enum.into(),
                start_time,
                block_name.clone(),
                ScheduleAction::StartBlock,
            ));

            // Add the end time
            let end_time = parse_time(&schedule.end);
            weekly_schedule.push(create_event(
                day_enum.into(),
                end_time,
                block_name,
                ScheduleAction::EndBlock,
            ));
        }
    }

    weekly_schedule.sort();
    application_state.lock().unwrap().schedule = weekly_schedule;
    application_state
}

fn parse_time(time_str: &str) -> NaiveTime {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        panic!("Invalid time format: {time_str}");
    }

    let hour = parts[0].parse::<u32>().expect("Invalid hour");
    let minute = parts[1].parse::<u32>().expect("Invalid minute");

    NaiveTime::from_hms_opt(hour, minute, 0).expect("Invalid time")
}

fn parse_day(day_str: &str) -> Result<Weekday, String> {
    DAY_MAP
        .iter()
        .find(|(d, _)| *d == day_str)
        .map(|(_, day)| day.clone())
        .ok_or_else(|| format!("Invalid day: {day_str}"))
}

pub fn update_block(application_state: &mut ApplicationState, block_name: &str, new_state: BlockState, state_tx: mpsc::Sender<()>) {
    if let Some(block) = application_state.blocks.get_mut(block_name) {
        block.block_state = new_state;
    } else {
        eprintln!("Block '{}' not found in application state", block_name);
    }

    let _ = state_tx.send(());
}

fn create_event(day: OrderableWeekday, time: NaiveTime, block: String, action: ScheduleAction) -> Event {
    Event {
        block,
        day,
        time,
        action,
    }
}
