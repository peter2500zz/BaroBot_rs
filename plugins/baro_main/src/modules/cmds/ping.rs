use std::sync::Arc;
use azalea_brigadier::prelude::*;
use kovi::event::RepliableEvent;

use crate::modules::cmds::AppCtx;


pub fn ping<T: RepliableEvent + Send + Sync>(disp: &mut CommandDispatcher<AppCtx<T>>) {
    disp.register(
        literal("ping")
            .executes(|ctx: &CommandContext<AppCtx<T>>| {
                let event = Arc::clone(&ctx.source.event);

                event.reply("pong!");

                0
            })
    );
}