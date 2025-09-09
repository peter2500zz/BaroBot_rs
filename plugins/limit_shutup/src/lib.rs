use std::{collections::HashMap, sync::Arc};

use kovi::{log::{info, warn}, PluginBuilder as plugin};
use config::load_config;
use serde::Deserialize;


#[derive(Deserialize)]
struct Config {
    limit_shutup: HashMap<i64, LimitShutupConfig>
}

#[derive(Deserialize)]
struct LimitShutupConfig {
    limit: usize,
    shutup_seconds: usize,
    reply: Option<String>
}

#[kovi::plugin]
async fn main() {
    if let Some(config) = load_config::<Config>() {
        let config = Arc::new(config.limit_shutup);

        for (group_id, config) in config.iter() {
            info!(
                "[Limit shutup] {} limited with {} chars, {} seconds{}", 
                group_id, 
                config.limit, 
                config.shutup_seconds, 
                if let Some(reply) = &config.reply { 
                    format!(", and reply with \"{}\"", reply) 
                } else {
                    String::new()
                }
            );
        }

        let bot = plugin::get_runtime_bot();

        plugin::on_group_msg(move |event|{
            let config = Arc::clone(&config);
            let bot = Arc::clone(&bot);

            async move {
                if let Some(config_for_this_group) = config.get(&event.group_id) {
                    if event.human_text.chars().count() >= config_for_this_group.limit {
                        info!(
                            "[Limit shutup] {}({}) in {} out of limit, banned for {} seconds", 
                            event.get_sender_nickname(),
                            event.user_id,
                            event.group_id,
                            config_for_this_group.shutup_seconds
                        );
                        bot.set_group_ban(event.group_id, event.user_id, config_for_this_group.shutup_seconds);

                        if let Some(reply) = &config_for_this_group.reply {
                            event.reply(reply);
                        }
                    }
                }
            }
        });
    } else {
        warn!("[Limit shutup] Mounted this plugin but find no valid config")
    }
}
