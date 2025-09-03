use azalea_brigadier::prelude::*;


type DispHandler = fn(&mut CommandDispatcher<AppCtx>);

pub struct Register {
    pub func: DispHandler,
}

impl Register {
    pub const fn new(func: DispHandler) -> Self {
        Self { func }
    }
}
pub struct AppCtx;



