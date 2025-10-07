mod cc;

use std::{collections::HashMap,  sync::Arc};
use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha12Rng};

use kovi::{event::GroupMsgEvent, tokio::sync::Mutex, PluginBuilder as plugin};
use config::load_config;
use serde::Deserialize;

use crate::cc::ChatCore;

#[derive(Deserialize)]
struct Config {
    randomly_ai: CCConfig
}

#[derive(Deserialize)]
struct CCConfig {
    api_key: String,
    api_base: String,
    max_token: u32,
    model: String,
    prompts: HashMap<String, String>,

    groups: HashMap<i64, CCGroupCfg>
}

#[derive(Deserialize)]
struct CCGroupCfg {
    prompt: String,
    memory: usize,
    chance: f32,
    max_try: Option<u32>
}

#[kovi::plugin]
async fn main() {
    let mut clients = HashMap::new();

    if let Some(config) = load_config::<Config>()
    {
        struct Storage {
            cc: ChatCore,
            cfg: CCGroupCfg,
            count: usize
        }

        for (group_id, group_config) in config.randomly_ai.groups {
            if let Some(prompt) = config.randomly_ai.prompts.get(&group_config.prompt)
            && let Ok(chatcore) = ChatCore::new(
                &config.randomly_ai.api_base, 
                &config.randomly_ai.api_key, 
                &config.randomly_ai.model, 
                config.randomly_ai.max_token, 
                prompt
            ) {
                clients.insert(group_id, Storage { cc: chatcore, cfg: group_config, count: 0 });
            }
        }

        let storage = Arc::new(Mutex::new(clients));

        plugin::on_group_msg(move |event| {
            let storage = Arc::clone(&storage);
            async move {
                let on_me = |event: &Arc<GroupMsgEvent>| -> bool {
                    if let Some(msg) = event.get("message").and_then(|v| v.as_array()) {
                        for unit in msg {
                            
                        // event.reply(format!("{:#?} {:#?}", unit.get("type").and_then(|v| v.as_str()).unwrap_or("?"), unit.get("data").and_then(|v| v.as_object()).and_then(|v| v.get("qq")).and_then(|v| v.as_str()).unwrap_or("?") ));
                            if let Some(_type) = unit.get("type").and_then(|v| v.as_str()) 
                                && _type == "at"
                                && let Some(_data) = unit.get("data").and_then(|v| v.as_object())
                                && let Some(_qq) = _data.get("qq").and_then(|v| v.as_str()) 
                                && _qq == event.self_id.to_string()
                            {
                                return true;
                            }
                        }
                    };

                    false
                };

                let mut guard = storage.lock().await;
                if let Some(for_this) = guard.get_mut(&event.group_id) && let Some(text) = &event.text {
                    // event.reply(&format!("{}说：{}", event.get_sender_nickname(), text));
                    if !on_me(&event) {
                        if for_this.cfg.max_try != Some(0) {
                            let mut rng = ChaCha12Rng::from_os_rng();

                            let delta = if let Some(max_try) = for_this.cfg.max_try {
                                (1.0 - for_this.cfg.chance) / (max_try) as f32
                            } else {
                                0.
                            };

                            let rnd = rng.random_range(0.0..=1.0);
                            if rnd > for_this.cfg.chance + delta * for_this.count as f32 {
                                // event.reply(format!("{:.2}/{:.2}/1.00", for_this.cfg.chance + delta * for_this.count as f32, rnd));
                                // for_this.cc.add_user_text(&format!("{}：{}", event.get_sender_nickname(), text), &event.get_sender_nickname()).unwrap();
                                for_this.cc.add_user_text(text, &event.get_sender_nickname()).unwrap();
                                for_this.count += 1;
                                return ;
                            }
                        }
                    }

                    for_this.count = 0;

                    // 因为包含系统提示词
                    if for_this.cc.messages.len() > for_this.cfg.memory {
                        for_this.cc.messages.remove(0);
                    }

                    let mut cc = for_this.cc.clone();
                    drop(guard);

                    // let response = cc.chat(&format!("{}：{}", event.get_sender_nickname(), text), &event.get_sender_nickname()).await.unwrap();
                    let response = cc.chat(text, &event.get_sender_nickname(), 1.3).await.unwrap();

                    event.reply(response);
                    let mut guard = storage.lock().await;
                    guard.get_mut(&event.group_id).unwrap().cc = cc;
                }
            }
        });
    }
}
