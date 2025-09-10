use kovi::{
    log::info, PluginBuilder as plugin
};

use std::{
    sync::Arc,
};

use crate::{
    GlobalState,
};


pub fn auto_shut_up(state: Arc<GlobalState>) {
    if let Some(config) = state.config.auto_shutup.clone() {
        for cfg in config {
            info!("[Auto shut up] {}: {} -> {}", cfg.group_id, cfg.start, cfg.end);
            let state_for_start = Arc::clone(&state);
            plugin::cron(&cfg.start, {
                move || {
                    let state = Arc::clone(&state_for_start);

                    async move {
                        let bot = Arc::clone(&state.bot);

                        info!("[Auto shut up] {}: SHUT UP!", cfg.group_id);
                        bot.set_group_whole_ban(cfg.group_id, true);
                    }
                }
            }).unwrap();

            let state_for_end = Arc::clone(&state);
            plugin::cron(&cfg.end, {
                move || {
                    let state = Arc::clone(&state_for_end);

                    async move {
                        let bot = Arc::clone(&state.bot);

                        info!("[Auto shut up] {}: NO SHUT UP.", cfg.group_id);
                        bot.set_group_whole_ban(cfg.group_id, false);
                    }
                }
            }).unwrap();
        }
    }
}
