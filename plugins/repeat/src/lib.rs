
use std::{collections::HashMap, sync::{Arc, LazyLock}};
use kovi::{tokio::{self, sync::Mutex}, Message, RuntimeBot};

use kovi::PluginBuilder as plugin;
use brigadier::{azalea_brigadier::prelude::*, get_command, inventory, register::{AppCtx, Register}};

static ACTIVED_GROUP: LazyLock<Arc<Mutex<HashMap<i64, (String, i32)>>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});


static LIMIT: LazyLock<Arc<Mutex<(i32, )>>> = LazyLock::new(|| {
    Arc::new(Mutex::new((2, )))
});

static CONTINUOUS: LazyLock<Arc<Mutex<(bool, )>>> = LazyLock::new(|| {
    Arc::new(Mutex::new((false, )))
});


#[kovi::plugin]
async fn main() {
    inventory::submit! {
        Register::new(|disp: &mut CommandDispatcher<AppCtx>| {
            disp.register(
                literal("repeat")
                .then(
                    literal("list")
                        .executes(ls_repeat)
                ).then(
                    literal("on")
                    .then(
                        argument("group_id", long())
                        .executes(repeat_with_arg)
                    ).then(
                        literal("here")
                        .requires(|ctx: &AppCtx| {
                            ctx.event.is_group()
                        })
                        .executes(repeat_at_this_group)
                    )
                ).then(
                    literal("off")
                    .then(
                        argument("group_id", long())
                        .executes(cancel_repeat_with_arg)
                    ).then(
                        literal("here")
                        .requires(|ctx: &AppCtx| {
                            ctx.event.is_group()
                        })
                        .executes(cancel_repeat_at_this_group)
                    )
                ).then(
                    literal("limit")
                    .then(
                        argument("limit", integer())
                        .executes(change_limit)
                    ).executes(get_limit)
                ).then(
                    literal("continuous")
                    .then(
                        argument("is_continuous", bool())
                        .executes(set_continuous)
                    ).executes(get_continuous)
                )
            );
        })
    }

    plugin::on_group_msg(async |event| {
        if let Some(mul_times) = ACTIVED_GROUP.lock().await.get_mut(&event.group_id) {
            if get_command(event.borrow_text().unwrap_or_default()).is_none() {

                if event.message.to_human_string() != mul_times.0 {
                    mul_times.0 = event.message.to_human_string();
                    mul_times.1 = 0;
                }

                mul_times.1 += 1;

                if LIMIT.lock().await.0.eq(&mul_times.1) {
                    if CONTINUOUS.lock().await.0 {
                        mul_times.1 = 0;
                    }
                    event.reply(event.message.clone());
                }
            }
        }
    });
}

fn get_continuous(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    tokio::spawn(async move {
        event.reply(format!("连续复读: {}", CONTINUOUS.lock().await.0));
    });

    0
}

fn set_continuous(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let is_continuous = get_bool(ctx, "is_continuous").unwrap_or_default();

    tokio::spawn(async move {
        let mut continuous = CONTINUOUS.lock().await;

        continuous.0 = is_continuous;

        if continuous.0 {
            event.reply(format!("已开启连续复读"));
        } else {
            event.reply(format!("已关闭连续复读"));
        }
    });

    0
}

fn get_limit(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    tokio::spawn(async move {
        event.reply(format!("当前复读阈值为 {}", LIMIT.lock().await.0));
    });

    0
}

fn change_limit(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let new_limit = get_integer(ctx, "limit").unwrap_or_default();

    tokio::spawn(async move {
        let mut limit = LIMIT.lock().await;
        limit.0 = new_limit;

        for (_, (_, count)) in ACTIVED_GROUP.lock().await.iter_mut() {
            *count = 0;
        }

        event.reply(format!("复读阈值已更改为 {}", new_limit));
    });

    0
}

async fn get_group_name(bot: &RuntimeBot, group_id: i64) -> Option<String> {
    if let Ok(value) = bot.get_group_info(group_id, true).await && 
    let Some(raw_name) = value.data.get("group_name") &&
    let Some(name) = raw_name.as_str() {
        Some(name.to_string())
    } else {
        None
    }
}

fn ls_repeat(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let bot = Arc::clone(&ctx.source.bot);

    tokio::spawn(async move {
        let actived_group = Arc::clone(&ACTIVED_GROUP);
        let actived_group = actived_group.lock().await;

        if actived_group.is_empty() {
            event.reply("当前不在任何群里复读");
        } else {
            let mut msg = Message::new();
            msg.push_text("当前正在以下群里复读:");

            for &group_id in actived_group.keys() {
                if let Some(group_name) = get_group_name(&bot, group_id).await {
                    msg.push_text(format!("\n{}({})", group_name, group_id));
                } else {
                    msg.push_text(format!("\n{}", group_id));
                }
            }

            event.reply(msg);
        }
    });

    0
}

fn cancel_repeat_at_this_group(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    if let Some(group_id) = event.group_id {
        tokio::spawn(async move {
            let actived_group = Arc::clone(&ACTIVED_GROUP);
            let mut actived_group = actived_group.lock().await;
            
            if actived_group.contains_key(&group_id) {
                actived_group.remove(&group_id);

                event.reply("已停止本群中的复读")
            } else {
                
                event.reply("并没有在本群中复读")
            }
        });
    } else {
        event.reply(format!("必须在群中发送或指定群ID"));
    }

    0
}

fn cancel_repeat_with_arg(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let bot = Arc::clone(&ctx.source.bot);
    let group_id = get_long(ctx, "group_id").unwrap_or_default();

    tokio::spawn(async move {
        let actived_group = Arc::clone(&ACTIVED_GROUP);
        let mut actived_group = actived_group.lock().await;
        
        if actived_group.contains_key(&group_id) {
            actived_group.remove(&group_id);

            if let Some(this_group_id) = event.group_id && this_group_id == group_id {
                event.reply(format!("已停止本群中的复读"));
            } else {
                if let Some(group_name) = get_group_name(&bot, group_id).await {
                    event.reply(format!("已停止在 {}({}) 中的复读", group_name, group_id));
                } else {
                    event.reply(format!("并没有在 {} 中复读", group_id));
                }
            }
        } else {
            
            event.reply("并没有在本群中复读")
        }
    });

    0
}

fn repeat_at_this_group(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);

    if let Some(group_id) = event.group_id {
        tokio::spawn(async move {
            let actived_group = Arc::clone(&ACTIVED_GROUP);
            let mut actived_group = actived_group.lock().await;

            if actived_group.contains_key(&group_id) {
                event.reply(format!("已经在本群中复读"));
            } else {
                actived_group.insert(group_id, (String::new(), 0));
                
                event.reply(format!("开始在本群中复读"));
            }
        });

        
    } else {
        event.reply(format!("必须在群中发送或指定群ID"));
    }

    0
}

fn repeat_with_arg(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let bot = Arc::clone(&ctx.source.bot);
    let group_id = get_long(ctx, "group_id").unwrap_or_default();

    tokio::spawn(async move {
        let actived_group = Arc::clone(&ACTIVED_GROUP);
        let mut actived_group = actived_group.lock().await;

        if actived_group.contains_key(&group_id) {

            if let Some(this_group_id) = event.group_id && this_group_id == group_id {
                event.reply(format!("已经在本群中复读"));
            } else {
                if let Some(group_name) = get_group_name(&bot, group_id).await {
                    event.reply(format!("已经在 {}({}) 中复读", group_name, group_id));
                } else {
                    event.reply(format!("已经在 {} 中复读", group_id));
                }
            }
        } else {
            actived_group.insert(group_id, (String::new(), 0));
            
            if let Some(this_group_id) = event.group_id && this_group_id == group_id {
                event.reply(format!("开始在本群中复读"));
            } else {
                if let Some(group_name) = get_group_name(&bot, group_id).await {
                    event.reply(format!("开始在 {}({}) 中复读", group_name, group_id));
                } else {
                    event.reply(format!("开始在 {} 中复读", group_id));
                }
            }
        }
    });

    0
}
