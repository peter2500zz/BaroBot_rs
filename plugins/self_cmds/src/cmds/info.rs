use std::sync::Arc;
use brigadier::{azalea_brigadier::prelude::*, register::AppCtx};
use kovi::{chrono::{FixedOffset, TimeZone, Utc}, Message};

pub fn self_info(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    let mut msg = Message::new();

    let ts: i64 = option_env!("BUILD_TIMESTAMP")
        .unwrap_or("0")
        .parse()
        .unwrap_or_default();

    let datetime = Utc
        .timestamp_opt(ts, 0)
        .single()
        .unwrap_or_default()
        .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap());
    
    msg.push_text(format!("BBGA(Baro Bot Great Again)\n"));
    msg.push_text(format!("Build at: {}\n", datetime.format("%y-%m-%d %H:%M:%S")));
    msg.push_text(format!("GPL v3.0 license\n"));
    msg.push_text(format!("Author: peter2500zz"));

    event.reply(msg);

    0
}
