use serde_yaml;
use std::fs;
use serde::de::DeserializeOwned;

use crate::CONFIG_PATH;

pub fn load_config<T>() -> Option<T>
where
    T: DeserializeOwned,
{
    let text = fs::read_to_string(CONFIG_PATH).ok()?;
    serde_yaml::from_str::<T>(&text).ok()
}


// #[derive(Deserialize, Default, Clone, Debug)]
// pub struct Config {
//     // config: Option<>
//     // pub live_reminder: Option<LiveConfig>,
//     // pub auto_shutup: Option<Vec<AutoShutUpConfig>>,
//     // pub timetable: Option<TimeTableConfig>
// }

// // #[derive(Deserialize, Default, Clone, Debug)]
// // pub struct LiveConfig {
// //     pub cron: String,
// //     pub reminder: HashMap<i64, HashSet<i64>>
// // }

// // #[derive(Deserialize, Default, Clone, Debug)]
// // pub struct AutoShutUpConfig {
// //     pub group_id: i64,
// //     pub start: String,
// //     pub end: String,
// // }

// // #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// // pub struct TimeTableConfig {
// //     pub receiver: i64,
// //     pub username: String,
// //     pub password: String,
// // }
// // Cargo.toml 需要：
// // serde = { version = "1", features = ["derive"] }
// // serde_yaml = "0.9"




// impl Config {
//     pub fn from_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
//         info!("[Config] Loading config from {}", path);
//         let config_content = fs::read_to_string(path)?;

//         let config: Self = serde_yaml::from_str(&config_content)?;

//         Ok(config)
//     }
// }

