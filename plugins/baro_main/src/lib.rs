mod modules;
mod config;

use kovi::{
    tokio::sync::Mutex, 
    PluginBuilder as plugin, RuntimeBot
};

use std::{
    time::Instant,
    sync::Arc,
    collections::HashMap,
};

use crate::{
    config::Config, 
    modules::*,
};


struct GlobalState {
    start_time: Instant,
    live_state: HashMap::<String, i32>,
    bot: Arc<RuntimeBot>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            live_state: HashMap::new(),
            bot: plugin::get_runtime_bot(),
        }
    }
}


struct PluginRegister {
    state: Arc<Mutex<GlobalState>>,
    config: Config,
}

impl PluginRegister {
    fn new(config: Config, state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            config,
            state,
        }
    }

    fn register(&self, reg_func: fn(config: Config, state: Arc<Mutex<GlobalState>>)) -> &Self {
        reg_func(self.config.clone(), Arc::clone(&self.state));

        self
    }
}


#[kovi::plugin]
async fn main() {
    let config = Config::from_yaml("config.yaml").unwrap_or_default();
    let state = Arc::new(Mutex::new(GlobalState::new()));

    let plg = PluginRegister::new(config, state);

    plg
    .register(admin_cmd)
    .register(live_reminder)
    .register(auto_shut_up)
    ;
}

