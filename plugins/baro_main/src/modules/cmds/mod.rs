mod ping;
mod random;
mod self_cmd;

use kovi::event::RepliableEvent;
use ping::ping;
use random::random;

use std::sync::Arc;
use azalea_brigadier::prelude::*;
use kovi::{event::AdminMsgEvent, tokio::sync::Mutex};
use kovi::PluginBuilder as plugin;

use crate::GlobalState;


const COMMAND_PREFIX: &str = "/";


#[derive(Clone)]
pub struct AppCtx<E>
where
    E: RepliableEvent + Send + Sync + 'static,
{
    event: Arc<E>,
    state: Arc<GlobalState>,
}

struct PluginBuilder<E>
where
    E: RepliableEvent + Send + Sync + 'static,
{
    disp: CommandDispatcher<AppCtx<E>>,
}

impl<E> PluginBuilder<E>
where
    E: RepliableEvent + Send + Sync + 'static,
{
    fn new() -> Self {
        Self {
            disp: CommandDispatcher::<AppCtx<E>>::new(),
        }
    }

    fn register(&mut self, reg_func: fn(&mut CommandDispatcher<AppCtx<E>>)) -> &mut Self {
        reg_func(&mut self.disp);
        self
    }

    fn build(self) -> CommandDispatcher<AppCtx<E>> {
        self.disp
    }
}

pub fn admin_cmd(state: Arc<GlobalState>) {
    let mut plg = PluginBuilder::new();

    plg
    .register(ping)
    .register(self_cmd::time)
    .register(random)
    ;

    let disp = Arc::new(Mutex::new(plg.build()));
    let state_for_this_closure = Arc::clone(&state);
    plugin::on_admin_msg(move |event| {
        let disp: Arc<Mutex<CommandDispatcher<AppCtx<AdminMsgEvent>>>> = Arc::clone(&disp);
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
