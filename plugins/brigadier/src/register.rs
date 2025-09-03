use std::sync::Arc;

use azalea_brigadier::prelude::*;
use kovi::event::AdminMsgEvent;

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
    pub event: Arc<AdminMsgEvent>
}

impl AppCtx {
    pub fn new(event: &Arc<AdminMsgEvent>) -> Self {
        Self { event: Arc::clone(event) }
    }
}



