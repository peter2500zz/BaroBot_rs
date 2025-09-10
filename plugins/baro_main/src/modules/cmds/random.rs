use std::sync::Arc;
use azalea_brigadier::prelude::*;
use kovi::{event::RepliableEvent, tokio, Message};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

use crate::modules::cmds::AppCtx;


#[derive(Deserialize)]
struct RandomFoxResponse {
    image: String,
}

#[derive(Deserialize)]
struct RandomCatResponse {
    url: String,
}

pub fn random<T: RepliableEvent + Send + Sync>(disp: &mut CommandDispatcher<AppCtx<T>>) {
    disp.register(
        literal("random")
        .then(
            literal("fox")
                .executes(|ctx: &CommandContext<AppCtx<T>>| {
                        let event = Arc::clone(&ctx.source.event);

                        tokio::spawn(async move {
                            match random_request::<RandomFoxResponse>("https://randomfox.ca/floof/").await {
                                Ok(response) => {
                                    let mut msg = Message::new();
                                    msg.push_image(&response.image);
                                    event.reply(msg);
                                },
                                Err(e) => {
                                    event.reply(format!("无法获取图片: {}", e));
                                }
                            }
                        });

                        0
                })
        )
        .then(
            literal("cat")
                .executes(|ctx: &CommandContext<AppCtx<T>>| {
                    let event = Arc::clone(&ctx.source.event);

                    tokio::spawn(async move {
                        match random_request::<Vec<RandomCatResponse>>("https://api.thecatapi.com/v1/images/search").await {
                            Ok(response) => {
                                for cat_response in response {
                                    let mut msg = Message::new();
                                    msg.push_image(&cat_response.url);
                                    event.reply(msg);

                                    break;
                                }
                            },
                            Err(e) => {
                                event.reply(format!("无法获取图片: {}", e));
                            }
                        }
                    });

                    0
                })
        )
    );
}

async fn random_request<T: DeserializeOwned>(url: &str) -> Result<T, reqwest::Error> {
    let client = Client::new();

    let body = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<T>()
        .await?;
    
    Ok(body)
}

