use std::{sync::{Arc, LazyLock}, time::Instant};

// use kovi::PluginBuilder as plugin;
use brigadier::{azalea_brigadier::prelude::*, inventory, register::{AppCtx, Register}};
use kovi::{chrono::{TimeZone, Utc}, Message};


pub static START: LazyLock<Instant> = LazyLock::new(Instant::now);

#[kovi::plugin]
async fn main() {
    let _ = START.elapsed();

    inventory::submit! {
        Register::new(|disp: &mut CommandDispatcher<AppCtx>| {

            disp.register(
                literal("self")
                .then(
                    literal("time")
                        .executes(self_time)
                )
                .then(
                    literal("info")
                    .executes(self_info)
                )
            );
        })
    }
}

fn self_time(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    event.reply(format!("已运行 {}", humantime::format_duration(START.elapsed())));

    0
}

fn self_info(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    let mut msg = Message::new();

    let datetime = Utc.timestamp_opt(option_env!("BUILD_TIMESTAMP").unwrap_or("0").parse().unwrap_or_default(), 0).single().unwrap_or_default();

    msg.push_text(format!("BBGA(Baro Bot Great Again)\n"));
    msg.push_text(format!("Build at: {}\n", datetime.format("%y-%m-%d %H:%M:%S")));
    msg.push_text(format!("GPL v3.0 license\n"));
    msg.push_text(format!("Author: peter2500zz"));

    event.reply(msg);

    0
}
