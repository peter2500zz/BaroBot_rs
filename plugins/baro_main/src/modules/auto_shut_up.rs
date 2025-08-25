use kovi::{
    tokio::sync::Mutex,
    PluginBuilder as plugin
};

use std::{
    sync::Arc,
};

use crate::{
    GlobalState,
    config::Config,
};


pub fn auto_shut_up(config: Config, state: Arc<Mutex<GlobalState>>) {
    for cfg in config.auto_shutup {
        let state_for_start = Arc::clone(&state);
        plugin::cron(&cfg.start, {
            move || {
                let state = Arc::clone(&state_for_start);

                async move {
                    let state = state.lock().await;
                    let bot = Arc::clone(&state.bot);

                    bot.set_group_whole_ban(cfg.group_id, true);
                }
            }
        }).unwrap();

        let state_for_end = Arc::clone(&state);
        plugin::cron(&cfg.end, {
            move || {
                let state = Arc::clone(&state_for_end);

                async move {
                    let state = state.lock().await;
                    let bot = Arc::clone(&state.bot);

                    bot.set_group_whole_ban(cfg.group_id, false);
                }
            }
        }).unwrap();
    }
}
