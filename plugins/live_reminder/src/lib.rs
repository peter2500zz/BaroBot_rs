mod room_info;

use std::sync::Arc;
use std::collections::{
    HashMap,
    HashSet,
};

use kovi::tokio;
use kovi::tokio::sync::Mutex;
use kovi::{
    log::info, Message, PluginBuilder as plugin
};

use brigadier::{azalea_brigadier::prelude::*, inventory, register::{AppCtx, Register}};
use config::load_config;
use serde::Deserialize;

use crate::room_info::get_live_status;


#[derive(Deserialize, Default, Clone, Debug)]
struct Config {
    live_reminder: LiveConfig,
}

#[derive(Deserialize, Default, Clone, Debug)]
struct LiveConfig {
    cron: String,
    reminder: HashMap<i64, HashSet<i64>>
}


struct LiveReminder {
    reminder: HashMap<i64, HashSet<i64>>,
    room_ids: HashSet<i64>
}

impl LiveReminder {
    fn new(live_config: LiveConfig) -> Self {
        let mut rooms = HashSet::<i64>::new();

        for (_, room_ids) in &live_config.reminder {
            for room_id in room_ids {
                rooms.insert(*room_id);
            }
        }

        Self {
            reminder: live_config.reminder,
            room_ids: rooms,
        }
    }
}

#[kovi::plugin]
async fn main() {
    inventory::submit! {
        Register::new(|disp: &mut CommandDispatcher<AppCtx>| {
            disp.register(
                literal("livequery")
                .then(
                    argument("room_id", integer())
                        .executes(live_query)
                )
            );
        })
    }

    let live_state: Arc<Mutex<HashMap::<String, i32>>> =  Arc::new(Mutex::new(HashMap::new()));
    let bot = plugin::get_runtime_bot();

    if let Some(config) = load_config::<Config>() {
        info!("[Live reminder] initializing rooms");

        let live_reminder = Arc::new(LiveReminder::new(config.live_reminder.clone()));

        plugin::cron(&config.live_reminder.cron.clone(), {
            move || {
                let live_reminder = Arc::clone(&live_reminder);
                let live_state = Arc::clone(&live_state);
                let bot = Arc::clone(&bot);

                async move {
                    if let Ok(response) = live_reminder.live_status().await {
                        let mut live_state = live_state.lock().await;

                        for (room_id, room) in response.data.by_room_ids {
                            let last_status = match live_state.get(&room_id).cloned() {
                                Some(s) => s,
                                None => {
                                    // ensure not send too much message at once
                                    info!("[Live reminder] room {} init with {}", room_id, room.live_status);
                                    live_state.insert(room_id, room.live_status);
                                    continue;
                                }
                            };

                            if room.live_status == last_status {
                                continue;
                            }

                            match room.live_status {
                                // 下播
                                0 => {
                                    info!("[Live reminder] room {} now not at stream any more.", room_id);

                                    for (group_id, room_ids) in &live_reminder.reminder {
                                        let room_id = room_id.parse::<i64>().unwrap_or_default();
                                        if room_ids.contains(&room_id) {
                                            bot.send_group_msg(*group_id, format!("{} 下播了", room.uname));
                                        }
                                    }
                                },

                                // 直播
                                1 => {
                                    let mut msg = Message::new();
                                    info!("[Live reminder] room {} now streaming.", room_id);

                                    msg.push_text(format!("{} {}\n", room.uname, if room.live_status == 1 { "正在直播" } else { "不在直播" }));
                                    msg.push_text(format!("{}\n", room.area_name));
                                    msg.push_image(&room.cover);
                                    msg.push_text(format!("{}\n", room.title));
                                    msg.push_text(room.live_url);

                                    for (group_id, room_ids) in &live_reminder.reminder {
                                        let room_id = room_id.parse::<i64>().unwrap_or_default();
                                        if room_ids.contains(&room_id) {
                                            bot.send_group_msg(*group_id, msg.clone());
                                        }
                                    }
                                },

                                // 轮播
                                2 => (),
                                _ => unreachable!()
                            }

                            live_state.insert(room_id, room.live_status);
                        }
                    }
                }
            }
        }).unwrap();
    } else {
        info!("[Live reminder] find no config, ignore");
    }
}

fn live_query(ctx: &CommandContext<AppCtx>) -> i32 {
    let event = Arc::clone(&ctx.source.event);
    let room_id = get_integer(ctx, "room_id").unwrap_or_default();
    
    tokio::spawn(async move {
        match get_live_status(&HashSet::from([room_id as i64])).await {
            Ok(response) =>  {
                for (room_id, room) in response.data.by_room_ids {
                    let mut msg = Message::new();
                    info!("[Live reminder] query {}", room_id);

                    let live_status = match room.live_status {
                        0 => "已下播",
                        1 => "正在直播",
                        2 => "轮播中",
                        _ => unreachable!()
                    };

                    msg.push_text(format!("{} {}\n", room.uname, live_status));
                    msg.push_text(format!("{}\n", room.area_name));
                    msg.push_image(&room.cover);
                    msg.push_text(format!("{}\n", room.title));
                    msg.push_text(room.live_url);

                    event.reply(msg);
                }
            },
            Err(e) => event.reply(format!("can not get live status: {e}")),
        }
    });

    0
}

