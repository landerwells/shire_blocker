use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub blocks: Vec<Block>,
    pub schedule: Vec<Schedule>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub default_action: String,
    pub log_violations: bool,
    pub notify_on_block: bool,
    pub strict_mode: Option<bool>, // optional
}

#[derive(Debug, Deserialize)]
pub struct Block {
    pub name: String,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Schedule {
    pub block: String,
    pub days: Vec<String>,
    pub start: String, // you can use chrono::NaiveTime if you want stricter typing
    pub end: String,
}

pub fn parse_config() -> Result<(), Box<dyn std::error::Error>> {
    let path = format!(
        "{}/.config/shire/shire.toml",
        std::env::var("HOME")?
    );

    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;

    // println!("{:#?}", config);

    Ok(())
}
