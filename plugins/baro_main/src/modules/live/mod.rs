mod room_info;

use std::{collections::{HashMap, HashSet}, sync::Arc};
use kovi::{
    tokio::sync::Mutex, 
    Message, PluginBuilder as plugin
};

use crate::{config::{Config, LiveConfig}, GlobalState};

// use std::sync::Arc;

// use kovi::{tokio, Message};

// use crate::AppCtx;


pub struct LiveReminder {
    pub reminder: HashMap<i64, HashSet<i64>>,
    pub room_ids: HashSet<i64>
}

impl LiveReminder {
    pub fn new(live_config: LiveConfig) -> Self {
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

pub fn live_reminder(config: Config, state: Arc<Mutex<GlobalState>>) {
    plugin::cron(&config.live.cron.clone(), {
        move || {
            let state = Arc::clone(&state);
            let live_reminder = Arc::new(LiveReminder::new(config.live.clone()));

            async move {
                if let Ok(response) = live_reminder.live_status().await {
                    let mut state = state.lock().await;

                    for (room_id, room) in response.data.by_room_ids {
                        let last_status = match state.live_state.get(&room_id).cloned() {
                            Some(s) => s,
                            None => {
                                // ensure not send too much message at once
                                println!("{} init with {}", room_id, room.live_status);
                                state.live_state.insert(room_id, room.live_status);
                                continue;
                            }
                        };

                        if room.live_status == last_status {
                            continue;
                        }

                        match room.live_status {
                            // 下播
                            0 => {
                                for (group_id, room_ids) in &live_reminder.reminder {
                                    let room_id = room_id.parse::<i64>().unwrap_or_default();
                                    if room_ids.contains(&room_id) {
                                        state.bot.send_group_msg(*group_id, format!("{} 下播了", room.uname));
                                    }
                                }
                            },

                            // 直播
                            1 => {
                                let mut msg = Message::new();

                                msg.push_text(format!("{} {}\n", room.uname, if room.live_status == 1 { "正在直播" } else { "不在直播" }));
                                msg.push_text(format!("{}\n", room.area_name));
                                msg.push_image(&room.cover);
                                msg.push_text(format!("{}\n", room.title));
                                msg.push_text(room.live_url);

                                for (group_id, room_ids) in &live_reminder.reminder {
                                    let room_id = room_id.parse::<i64>().unwrap_or_default();
                                    if room_ids.contains(&room_id) {
                                        state.bot.send_group_msg(*group_id, msg.clone());
                                    }
                                }
                            },

                            // 轮播
                            2 => (),
                            _ => unreachable!()
                        }

                        state.live_state.insert(room_id, room.live_status);
                    }
                }
            }
        }
    }).unwrap();
}

