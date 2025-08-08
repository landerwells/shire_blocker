use std::fs;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // pub settings: Settings,
    pub blocks: Vec<Block>,
    // pub schedule: Vec<Schedule>,
}

// #[derive(Debug, Deserialize)]
// pub struct Settings {
//     pub default_action: String,
//     pub log_violations: bool,
//     pub notify_on_block: bool,
//     pub strict_mode: Option<bool>, // optional
// }

#[derive(PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub name: String,
    pub active_by_default: Option<bool>,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
}

// I leave scheduling in for now, but I am not going to work on that feature for
// the first release.
// #[derive(Debug, Deserialize)]
// pub struct Schedule {
//     pub block: String,
//     pub days: Vec<String>,
//     pub start: String,
//     pub end: String,
// }

pub fn parse_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = format!(
        "{}/.config/shire/shire.toml",
        std::env::var("HOME")?
    );

    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}


// Need to properly test the configuration and parsing.


// Generally I want good error messages, and I think that I will output errors
// to a single location just like skhd does.



// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_name() {
//
//     }
// }
