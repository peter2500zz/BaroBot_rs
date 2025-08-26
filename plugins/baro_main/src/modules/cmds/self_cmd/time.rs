use std::sync::Arc;
use azalea_brigadier::prelude::*;
use kovi::{event::RepliableEvent, tokio};

use crate::modules::cmds::AppCtx;


pub fn time<T: RepliableEvent + Send + Sync>(disp: &mut CommandDispatcher<AppCtx<T>>) {
    disp.register(
        literal("self")
        .then(
            literal("time")
                .executes(|ctx: &CommandContext<AppCtx<T>>| {
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
