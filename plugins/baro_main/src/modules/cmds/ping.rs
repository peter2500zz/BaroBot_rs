use std::sync::Arc;
use azalea_brigadier::prelude::*;

use crate::modules::cmds::AppCtx;


pub fn ping(disp: &mut CommandDispatcher<AppCtx>) {
    disp.register(
        literal("ping")
            .executes(|ctx: &CommandContext<AppCtx>| {
                let event = Arc::clone(&ctx.source.event);

                event.reply("pong!");

                0
            })
    );
}