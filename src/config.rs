use std::fs;
use serde::Deserialize;
use serde::Serialize;

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

// I don't really understand this return type
pub fn parse_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = format!(
        "{}/.config/shire/shire.toml",
        std::env::var("HOME")?
    );

    // Could make sure that all blocks in the schedule are blocks that actually
    // exist.
    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_name() {
//
//     }
// }
