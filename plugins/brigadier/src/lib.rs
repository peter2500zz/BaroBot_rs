mod register;

use std::sync::Arc;

use azalea_brigadier::prelude::*;
use kovi::PluginBuilder as plugin;
use inventory;

use crate::register::{AppCtx, Register};


const COMMAND_PREFIX: &str = "/";

// inv会收集其他所有使用Register并注册的库
// 方便实现插拔
inventory::collect!(Register);


#[kovi::plugin]
async fn main() {
    // 先定义非Arc的disp方便注册
    let mut disp = CommandDispatcher::<AppCtx>::new();

    for register in inventory::iter::<Register> {
        (register.func)(&mut disp)
    }

    let disp = Arc::new(disp);

    plugin::on_admin_msg(move |event| {
        let disp = Arc::clone(&disp);

        async move {

            if let Some(command) = is_command(event.borrow_text().unwrap_or_default()) {
                let ret = {
                    disp.execute(command, AppCtx)
                };

                match ret {
                    Ok(_) => (),
                    Err(e) => {
                        event.reply(e.message());
                    },
                }
            }
        }
    });
}


fn is_command(raw_msg: &str) -> Option<String> {
    if raw_msg.starts_with(COMMAND_PREFIX) {
        Some(raw_msg.chars().skip(1).collect::<String>())
    } else {
        None
    }
}

