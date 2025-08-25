use std::sync::Arc;
use azalea_brigadier::prelude::*;
use kovi::tokio;

use crate::modules::cmds::AppCtx;


pub fn time(disp: &mut CommandDispatcher<AppCtx>) {
    disp.register(
        literal("self")
        .then(
            literal("time")
                .executes(|ctx: &CommandContext<AppCtx>| {
                    let event = Arc::clone(&ctx.source.event);
                    let state = Arc::clone(&ctx.source.state);

                    tokio::spawn(async move {
                        let state= state.lock().await;

                        event.reply(format!("已运行 {}", humantime::format_duration(state.start_time.elapsed())));
                    });

                    0
                })
        )
    );

}
