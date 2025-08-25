use std::collections::HashMap;

use reqwest::header::*;
use reqwest::Client;
use serde::Deserialize;

use crate::modules::live::LiveReminder;


#[derive(Deserialize)]
pub struct Resp {
    pub data: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub by_room_ids: HashMap<String, Room>,
}

#[derive(Deserialize)]
pub struct Room {
    pub live_url: String,
    pub title: String,
    pub area_name: String,
    pub uname: String,
    pub cover: String,
    pub live_status: i32,
}

impl LiveReminder {
    pub async fn live_status(&self) -> Result<Resp, reqwest::Error> {
        let mut query = vec![("req_biz", "web_room_componet".to_string())];

        for room_id in &self.room_ids {
            query.push(("room_ids", room_id.to_string()));
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
            ),
        );

        let client = Client::builder().default_headers(headers).build()?;

        let body = client
            .get("https://api.live.bilibili.com/xlive/web-room/v1/index/getRoomBaseInfo")
            .query(&query)
            .send()
            .await?
            .error_for_status()?
            .json::<Resp>()
            .await?;

        Ok(body)
    }
}


