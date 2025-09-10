use std::sync::Arc;
use brigadier::{azalea_brigadier::prelude::*, register::AppCtx};



pub fn self_time(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    event.reply(format!("已运行 {}", humantime::format_duration(crate::START.elapsed())));

    0
}