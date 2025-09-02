mod modules;
mod config;

use kovi::{
    log::info, tokio::sync::Mutex, PluginBuilder as plugin, RuntimeBot
};
use reqwest::Client;

use std::{
    time::Instant,
    sync::Arc,
    collections::HashMap,
};

use crate::{
    config::Config, 
    modules::*,
};


const INTRO: &str = r#"[Baro] Printing intro...
    ____  ___    ____  ____              
   / __ )/   |  / __ \/ __ \   __________
  / __  / /| | / /_/ / / / /  / ___/ ___/
 / /_/ / ___ |/ _, _/ /_/ /  / /  (__  ) 
/_____/_/  |_/_/ |_|\____/  /_/  /____/  
"#;

struct GlobalState {
    bot: Arc<RuntimeBot>,
    start_time: Instant,

    live_state: HashMap::<String, i32>,

    tt_client: Option<Client>,
    // schedule_cache: None,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            live_state: HashMap::new(),
            bot: plugin::get_runtime_bot(),
            tt_client: None
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
    info!("[Baro] Registering plugins...");

    let config = Config::from_yaml("config.yaml").unwrap_or_default();
    let state = Arc::new(Mutex::new(GlobalState::new()));

    let plg = PluginRegister::new(config, state);

    plg
    .register(admin_cmd)
    .register(live_reminder)
    .register(auto_shut_up)
    ;

    info!("{INTRO}");
}

