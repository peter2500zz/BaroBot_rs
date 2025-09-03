mod modules;
mod config;

use kovi::{
    log::info, tokio::sync::Mutex, PluginBuilder as plugin, RuntimeBot
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use reqwest::{header::{HeaderMap, HeaderValue, USER_AGENT}, Client};

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
    config: Config,

    bot: Arc<RuntimeBot>,
    start_time: Instant,
    memory_db_pool: Pool<SqliteConnectionManager>,

    live_state: Mutex<HashMap::<String, i32>>,

    tt_client: Arc<Client>,
    // schedule_cache: None,
}

impl GlobalState {
    pub fn new(config: Config) -> Self {
        let mut header = HeaderMap::new();
        header.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));


        let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(20))
        .cookie_store(true)

        .default_headers(header)
        .build().unwrap();

        let manager = SqliteConnectionManager::memory()

        .with_init(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE schedule (
                    week     INTEGER,
                    weekday  TEXT,
                    queue    INTEGER,
                    term    INTEGER,
                    name     TEXT
                )
                ;"#,
            )
        });

        let pool = r2d2::Pool::builder()
        .max_size(1)
        .min_idle(Some(1))
        .max_lifetime(None)
        .idle_timeout(None)
        .build(manager).unwrap();

        Self {
            config,

            bot: plugin::get_runtime_bot(),
            start_time: Instant::now(),
            memory_db_pool: pool,

            live_state: Mutex::new(HashMap::new()),

            tt_client: Arc::new(client)
        }
    }
}

struct PluginRegister {
    state: Arc<GlobalState>,
}

impl PluginRegister {
    fn new(state: Arc<GlobalState>) -> Self {
        Self {
            state,
        }
    }

    fn register(&self, reg_func: fn(state: Arc<GlobalState>)) -> &Self {
        reg_func(Arc::clone(&self.state));

        self
    }
}

pub fn d() {
    use azalea_brigadier::prelude::*;
}

#[kovi::plugin]
async fn main() {
    info!("[Baro] Registering plugins...");

    let config = Config::from_yaml("config.yaml").unwrap_or_default();
    let state = Arc::new(GlobalState::new(config));

    let plg = PluginRegister::new(state);

    plg
    .register(admin_cmd)
    .register(live_reminder)
    .register(auto_shut_up)
    ;

    info!("{INTRO}");
}

