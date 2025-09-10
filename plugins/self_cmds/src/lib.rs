mod cmds;

use std::{sync::LazyLock, time::Instant};

use brigadier::{azalea_brigadier::prelude::*, inventory, register::{AppCtx, Register}};


static START: LazyLock<Instant> = LazyLock::new(Instant::now);

#[kovi::plugin]
async fn main() {
    let _ = START.elapsed();

    inventory::submit! {
        Register::new(|disp: &mut CommandDispatcher<AppCtx>| {

            // self 命令集
            disp.register(
                literal("self")
                .then(
                    literal("time")
                        .executes(cmds::time)
                )
                .executes(cmds::info)
            );

            // send 命令集
            disp.register(
                literal("send")
                .then(
                    literal("group")
                    .then(
                        argument("group_id", long())
                        .then(
                            argument("msg", greedy_string())
                            .executes(cmds::send::group)
                        )
                    )
                )
                .then(
                    argument("user_id", long())
                    .then(
                        argument("msg", greedy_string())
                        .executes(cmds::send::private)
                    )
                )
            );
        })
    }
}




