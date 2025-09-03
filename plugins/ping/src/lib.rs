use std::sync::Arc;

// use kovi::PluginBuilder as plugin;
use brigadier::{azalea_brigadier::prelude::*, inventory, register::{AppCtx, Register}};

#[kovi::plugin]
async fn main() {
    inventory::submit! {
        Register::new(|disp: &mut CommandDispatcher<AppCtx>| {
            disp.register(
                literal("ping")
                    .executes(ping)
            );
        })
    }
}

fn ping(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    event.reply("pong!");

    0
}
