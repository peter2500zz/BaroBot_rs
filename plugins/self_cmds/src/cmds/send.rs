use std::sync::Arc;
use brigadier::{azalea_brigadier::prelude::*, register::AppCtx};
use kovi::tokio;


pub fn group(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let bot = Arc::clone(&ctx.source.bot);

    let group_id = get_long(ctx, "group_id").unwrap_or_default();
    let msg = get_string(ctx, "msg").unwrap_or_default();
    
    tokio::spawn(async move {
        let send_result = bot.send_group_msg_return(group_id, msg).await;

        match send_result {
            Ok(msg_id) => {
                event.reply(format!("发送成功，消息ID: {}", msg_id));
            },
            Err(e) => {
                event.reply(format!("发送失败: {}", e));
            }
        }
    });

    0
}

pub fn private(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let bot = Arc::clone(&ctx.source.bot);

    let user_id = get_long(ctx, "user_id").unwrap_or_default();
    let msg = get_string(ctx, "msg").unwrap_or_default();

    tokio::spawn(async move {
        let send_result = bot.send_private_msg_return(user_id, msg).await;

        match send_result {
            Ok(msg_id) => {
                event.reply(format!("发送成功，消息ID: {}", msg_id));
            },
            Err(e) => {
                event.reply(format!("发送失败: {}", e));
            }
        }
    });

    0
}
