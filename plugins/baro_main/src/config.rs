use serde::{Deserialize};
use serde_yaml;
use std::fs;
use std::collections::{HashMap, HashSet};


#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub live: LiveConfig,
    pub auto_shutup: Vec<AutoShutUpConfig>
}

#[derive(Deserialize, Default, Clone)]
pub struct LiveConfig {
    pub cron: String,
    pub reminder: HashMap<i64, HashSet<i64>>
}

#[derive(Deserialize, Default, Clone)]
pub struct AutoShutUpConfig {
    pub group_id: i64,
    pub start: String,
    pub end: String,
}


impl Config {
    pub fn from_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;

        let config: Self = serde_yaml::from_str(&config_content)?;

        Ok(config)
    }
}

