use std::sync::Arc;

use azalea_brigadier::prelude::*;
use kovi::{event::AdminMsgEvent, RuntimeBot};

type DispHandler = fn(&mut CommandDispatcher<AppCtx>);

pub struct Register {
    pub func: DispHandler,
}

impl Register {
    pub const fn new(func: DispHandler) -> Self {
        Self { func }
    }
}

pub struct AppCtx {
    pub event: Arc<AdminMsgEvent>,
    pub bot: Arc<RuntimeBot>
}

impl AppCtx {
    pub fn new(event: &Arc<AdminMsgEvent>, bot: &Arc<RuntimeBot>) -> Self {
        Self { 
            event: Arc::clone(event),
            bot: Arc::clone(bot)
        }
    }
}



