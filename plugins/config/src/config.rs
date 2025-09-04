use kovi::log::info;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::collections::{HashMap, HashSet};


#[derive(Deserialize, Default, Clone, Debug)]
pub struct Config {
    pub live_reminder: Option<LiveConfig>,
    pub auto_shutup: Option<Vec<AutoShutUpConfig>>,
    pub timetable: Option<TimeTableConfig>
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct LiveConfig {
    pub cron: String,
    pub reminder: HashMap<i64, HashSet<i64>>
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct AutoShutUpConfig {
    pub group_id: i64,
    pub start: String,
    pub end: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TimeTableConfig {
    pub receiver: i64,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn from_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("[Config] Loading config from {}", path);
        let config_content = fs::read_to_string(path)?;

        let config: Self = serde_yaml::from_str(&config_content)?;

        Ok(config)
    }
}

