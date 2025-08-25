mod ping;
mod random;
mod self_cmd;

use ping::ping;
use random::random;

use std::sync::Arc;
use azalea_brigadier::prelude::*;
use kovi::{event::AdminMsgEvent, tokio::sync::Mutex};
use kovi::PluginBuilder as plugin;

use crate::{config::Config, GlobalState};


const COMMAND_PREFIX: &str = "/";


#[derive(Clone)]
pub struct AppCtx {
    event: Arc<AdminMsgEvent>,
    state: Arc<Mutex<GlobalState>>,
}


struct PluginBuilder {
    disp: CommandDispatcher<AppCtx>,
}

impl PluginBuilder {
    fn new() -> Self {
        Self {
            disp: CommandDispatcher::<AppCtx>::new(),
        }
    }

    fn register(&mut self, reg_func: fn(&mut CommandDispatcher<AppCtx>)) -> &mut Self {
        reg_func(&mut self.disp);

        self
    }

    fn build(self) -> CommandDispatcher<AppCtx> {
        self.disp
    }
}

pub fn admin_cmd(_: Config, state: Arc<Mutex<GlobalState>>) {
    let mut plg = PluginBuilder::new();

    plg
    .register(ping)
    .register(self_cmd::time)
    .register(random)
    ;

    let disp = Arc::new(Mutex::new(plg.build()));
    let state_for_this_closure = Arc::clone(&state);
    plugin::on_admin_msg(move |event| {
        let disp = Arc::clone(&disp);
        let state = Arc::clone(&state_for_this_closure);

        async move {
            if let Some(command) = is_command(event.borrow_text().unwrap_or_default()) {
                let app = AppCtx { event: Arc::clone(&event), state };

                let ret = {
                    let disp = disp.lock();
                    disp.await.execute(command, app)
                };

                match ret {
                    Ok(_) => (),
                    Err(e) => {
                        event.reply(e.message());
                    },
                }
            }
        }
    });
}

fn is_command(raw_msg: &str) -> Option<String> {
    if raw_msg.starts_with(COMMAND_PREFIX) {
        Some(raw_msg.chars().skip(1).collect::<String>())
    } else {
        None
    }
}
