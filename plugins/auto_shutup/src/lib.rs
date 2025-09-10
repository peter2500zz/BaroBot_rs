use std::sync::Arc;

use config::load_config;
use kovi::{log::info, PluginBuilder as plugin};
use serde::Deserialize;


#[derive(Deserialize, Default, Clone, Debug)]
struct Config {
    auto_shutup: Vec<AutoShutUpConfig>,
}

#[derive(Deserialize, Default, Clone, Debug)]
struct AutoShutUpConfig {
    group_id: i64,
    start: String,
    end: String,
}

#[kovi::plugin]
async fn main() {
    let bot = plugin::get_runtime_bot();

    if let Some(config) = load_config::<Config>() {
        for cfg in config.auto_shutup {
            info!("[Auto shut up] {}: {} -> {}", cfg.group_id, cfg.start, cfg.end);

            let bot_for_start = Arc::clone(&bot);
            plugin::cron(&cfg.start, {
                move || {
                    let bot = Arc::clone(&bot_for_start);

                    async move {
                        let bot = Arc::clone(&bot);

                        info!("[Auto shut up] {}: SHUT UP!", cfg.group_id);
                        bot.set_group_whole_ban(cfg.group_id, true);
                    }
                }
            }).unwrap();

            let bot_for_end = Arc::clone(&bot);
            plugin::cron(&cfg.end, {
                move || {
                    let bot = Arc::clone(&bot_for_end);

                    async move {
                        let bot = Arc::clone(&bot);

                        info!("[Auto shut up] {}: NO SHUT UP.", cfg.group_id);
                        bot.set_group_whole_ban(cfg.group_id, false);
                    }
                }
            }).unwrap();
        }
    }
}
