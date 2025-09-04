pub mod register;

use std::sync::Arc;

// 其他包不需要cargo add也能用
pub use azalea_brigadier;
pub use inventory;
use azalea_brigadier::prelude::*;
use kovi::{log::info, PluginBuilder as plugin};

use crate::register::{AppCtx, Register};


/// 命令前缀
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

    // 包装为Arc
    let disp = Arc::new(disp);

    plugin::on_admin_msg(move |event| {
        // 克隆闭包用的引用
        let disp = Arc::clone(&disp);

        async move {
            if let Some(command) = get_command(event.borrow_text().unwrap_or_default()) {
                info!("[Brigadier] received a command: {}", command);

                let ret = {
                    disp.execute(command, AppCtx::new(&event))
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

/// 根据 `COMMAND_PREFIX` 判断是否是命令</br >
/// 并提取出命令部分
fn get_command(raw_msg: &str) -> Option<String> {
    if raw_msg.starts_with(COMMAND_PREFIX) {
        Some(raw_msg.chars().skip(1).collect::<String>())
    } else {
        None
    }
}

