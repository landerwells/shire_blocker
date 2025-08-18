use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub blocks: Vec<Block>,
    pub schedule: Vec<Schedule>,
}

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub name: String,
    pub active_by_default: Option<bool>,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Schedule {
    pub block: String,
    pub days: Vec<String>,
    pub start: String,
    pub end: String,
}

pub fn parse_config(config_path: Option<String>) -> Result<Config, Box<dyn std::error::Error>> {
    let path = match config_path {
        Some(custom_path) => custom_path,
        None => format!("{}/.config/shire/shire.toml", std::env::var("HOME")?),
    };

    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;

    validate_blocks_exist(&config)?;
    validate_schedule_times(&config)?;

    Ok(config)
}

fn validate_blocks_exist(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let block_names: std::collections::HashSet<&String> =
        config.blocks.iter().map(|b| &b.name).collect();

    for schedule in &config.schedule {
        if !block_names.contains(&schedule.block) {
            return Err(format!(
                "Schedule references non-existent block: '{}'.",
                schedule.block
            )
            .into());
        }
    }

    Ok(())
}

fn validate_day(day: &str) -> Result<(), String> {
    let valid_days = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    let day_lower = day.to_lowercase();

    if !valid_days.contains(&day_lower.as_str()) {
        return Err(format!(
            "Invalid day: '{}'. Valid days are: {}",
            day,
            valid_days.join(", ")
        ));
    }

    Ok(())
}

fn validate_schedule_times(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    for schedule in &config.schedule {
        for day in &schedule.days {
            validate_day(day)?;
        }

        validate_time(&schedule.start).map_err(|e| {
            format!(
                "Invalid start time in schedule for block '{}': {}",
                schedule.block, e
            )
        })?;
        validate_time(&schedule.end).map_err(|e| {
            format!(
                "Invalid end time in schedule for block '{}': {}",
                schedule.block, e
            )
        })?;
    }

    Ok(())
}

fn validate_time(time_str: &str) -> Result<(i32, i32), String> {
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

// The validation can be removed from this function in favor of validating in
// config.rs, and instead this can just be to parse out the hour and minute.
// Should be moved to state.rs
// fn parse_time(time_str: &str) -> Result<(i32, i32), String> {
//     if time_str.len() < 5 || !time_str.contains(':') {
//         return Err(format!("Invalid time format: {time_str}"));
//     }
//
//     let parts: Vec<&str> = time_str.split(':').collect();
//     if parts.len() != 2 {
//         return Err(format!("Invalid time format: {time_str}"));
//     }
//
//     let hour = parts[0]
//         .parse::<i32>()
//         .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
//     let minute = parts[1]
//         .parse::<i32>()
//         .map_err(|_| format!("Invalid minute: {}", parts[1]))?;
//
//     if !(0..=23).contains(&hour) {
//         return Err(format!("Hour out of range (0-23): {hour}"));
//     }
//     if !(0..=59).contains(&minute) {
//         return Err(format!("Minute out of range (0-59): {minute}"));
//     }
//
//     Ok((hour, minute))
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_day_valid_days() {
        assert!(validate_day("Mon").is_ok());
        assert!(validate_day("Tue").is_ok());
        assert!(validate_day("Wed").is_ok());
        assert!(validate_day("Thu").is_ok());
        assert!(validate_day("Fri").is_ok());
        assert!(validate_day("Sat").is_ok());
        assert!(validate_day("Sun").is_ok());
    }

    #[test]
    fn test_validate_day_invalid_days() {
        assert!(validate_day("monday").is_err());
        assert!(validate_day("xyz").is_err());
        assert!(validate_day("").is_err());
        assert!(validate_day("123").is_err());
    }

    #[test]
    fn test_validate_day_error_message() {
        let result = validate_day("invalid");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Invalid day: 'invalid'"));
        assert!(error.contains("Valid days are:"));
    }
}
