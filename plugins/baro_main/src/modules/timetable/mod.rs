use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OptionalExtension;
use serde::{self, Deserialize, Serialize};

use azalea_brigadier::prelude::*;
use kovi::{event::RepliableEvent, tokio::{self, sync::Mutex}, PluginBuilder as plugin};
use reqwest::{cookie, header::{HeaderMap, HeaderValue, USER_AGENT}, Client};

use crate::{config::Config, modules::cmds::AppCtx, GlobalState};



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Weekday {
    Mon, Tue, Wed, Thu, Fri, Sat, Sun,
    /// 承载原始的非法输入
    Invalid(String),
}

impl Weekday {
    fn from(n: String) -> Self {
        match n.as_str() {
            "1" => Weekday::Mon,
            "2" => Weekday::Tue,
            "3" => Weekday::Wed,
            "4" => Weekday::Thu,
            "5" => Weekday::Fri,
            "6" => Weekday::Sat,
            "7" => Weekday::Sun,
            _ => Weekday::Invalid(n),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Weekday::Mon => "Monday".to_string(),
            Weekday::Tue => "Tuesday".to_string(),
            Weekday::Wed => "Wednesday".to_string(),
            Weekday::Thu => "Thursday".to_string(),
            Weekday::Fri => "Friday".to_string(),
            Weekday::Sat => "Saturday".to_string(),
            Weekday::Sun => "Sunday".to_string(),
            Weekday::Invalid(n) => n.clone(),
        }
    }
}

#[derive(Deserialize, Default, Debug)]
struct TTRoot {
    data: TTData
}

#[derive(Deserialize, Default, Debug)]
struct TTData {
    #[serde(rename = "Rows")]
    rows: Vec<TTUnit>
}

#[derive(Deserialize, Debug)]
pub struct TTUnit {
    #[serde(rename = "XN")]
    #[serde(deserialize_with = "string_to_i32")]
    pub term: i32,

    #[serde(rename = "ZCFX")]
    #[serde(deserialize_with = "dot_integer")]
    pub weeks: Vec<i32>,

    #[serde(rename = "JCZ")]
    #[serde(deserialize_with = "get_week_day")]
    pub weekday: Weekday,

    #[serde(rename = "JCFX")]
    #[serde(deserialize_with = "dot_integer")]
    pub queues: Vec<i32>,

    #[serde(rename = "KCMC")]
    pub name: String,
}

fn dot_integer<'de, D>(deserializer: D) -> Result<Vec<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.split(',')
        .map(|item| item.trim().parse::<i32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(serde::de::Error::custom)
}

fn string_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.parse::<i32>() {
        Ok(num) => Ok(num),
        Err(e) => Err(serde::de::Error::custom(e))
    }
}

fn get_week_day<'de, D>(deserializer: D) -> Result<Weekday, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    
    Ok(Weekday::from(s.chars().nth(1).unwrap_or_default().to_string()))
}

struct Class {
    name: String,
    location: String,
    teacher: String,
}

pub fn cmd<T: RepliableEvent + Send + Sync>(disp: &mut CommandDispatcher<AppCtx<T>>) {
    disp.register(
        literal("tt")
        .then(
            literal("get")
            .executes(|ctx| {
                0
            })
            .then(
                argument("week", integer())
                .executes(|ctx| {
                    0
                })
                .then(
                    argument("week_day", integer())
                )
            )
        )
    );
}

pub fn init(state: Arc<GlobalState>) {
    if let Some(config) = state.config.timetable.clone() {

        tokio::spawn(async move {
            let client = Arc::clone(&state.tt_client);
            let result = get_schedule(client, &config).await;

            if let Ok(data) = result && let Ok(conn) = state.memory_db_pool.get() {
                if let Err(_) = set_schedule(conn, data) {
                    return;
                }

                let cron = [
                    "0 50 7 * * *",
                    "0 40 8 * * *",
                    "0 40 9 * * *",
                    "0 30 10 * * *",
                    "0 0 13 * * *",
                    "0 45 13 * * *",
                    "0 35 14 * * *",
                    "0 20 15 * * *",
                    "0 30 17 * * *",
                    "0 20 18 * * *",
                    "0 10 19 * * *",
                    "0 0 20 * * *",
                ];

                for (index, &cron) in cron.iter().enumerate() {
                    plugin::cron(cron, {
                        let state = Arc::clone(&state);
                        move || {
                            let state = Arc::clone(&state);

                            async move {
                                let bot = Arc::clone(&state.bot);

                                let conn = match state.memory_db_pool.get() {
                                    Ok(conn) => conn,
                                    Err(e) => {
                                        bot.send_private_msg(config.receiver, format!("Can not get conn: {}", e));
                                        return;
                                    }
                                };

                                
                            }
                        }
                    }).unwrap();
                }
            }
        });
    }
}

async fn get_schedule<T: Serialize + ?Sized>(client: Arc<Client>, account: &T) -> Result<Vec<TTUnit>, reqwest::Error> {
    client
    .post("https://sso.jsit.edu.cn/sso/login")
    .form(account)
    .send().await?;

    client
    .get("https://i.jsit.edu.cn/sso/login")
    .send()
    .await?;

    let schedule = client.get("https://i.jsit.edu.cn/api/node/168")
    .send()
    .await?
    .json::<TTRoot>()
    .await?;

    Ok(schedule.data.rows)
}

fn set_schedule(conn: r2d2::PooledConnection<SqliteConnectionManager>, data: Vec<TTUnit>) -> Result<(), rusqlite::Error> {
    for unit in data {
        for week in unit.weeks.to_vec() {
            for queue in unit.queues.to_vec() {
                conn.execute(
                    r#"
                    INSERT INTO schedule (
                        week, 
                        weekday, 
                        queue,
                        term,
                        name
                    ) VALUES (
                        ?1, 
                        ?2, 
                        ?3,
                        ?4,
                        ?5
                    )
                    ;"#, 
                    [
                        &week.to_string(),
                        &unit.weekday.to_string(),
                        &queue.to_string(),
                        &unit.term.to_string(),
                        &unit.name
                    ]
                )?;
            }
        }
    }

    Ok(())
}

fn get_class(conn: r2d2::PooledConnection<SqliteConnectionManager>, week: i32, weekday: Weekday, term: i32, queue: i32) -> Option<String> {
    conn.query_row(
        r#"
        SELECT name
        FROM schedule
        WHERE week = ?1
        AND weekday = ?2
        AND term = ?3
        AND queue = ?4
        LIMIT 1
        ;"#,
        [
            week.to_string(),
            weekday.to_string(),
            term.to_string(),
            queue.to_string()
        ],
        |row| row.get::<_, String>(0),
    ).optional().unwrap()
}

// #[tokio::test]
// async fn g() {
//     let manager = SqliteConnectionManager::memory()

//     .with_init(|conn| {
//         conn.busy_timeout(Duration::from_secs(5))?; // 锁冲突时自动等待
//         conn.execute_batch(
//             r#"
//             CREATE TABLE schedule (
//                 week     INTEGER,
//                 weekday  TEXT,
//                 queue    INTEGER,
//                 term    INTEGER,
//                 name     TEXT
//             )
//             ;"#,
//         )
//     });

//     let pool = r2d2::Pool::builder()
//     .max_size(1)
//     .min_idle(Some(1))
//     .max_lifetime(None)
//     .idle_timeout(None)
//     .build(manager).unwrap();

//     let mut header = HeaderMap::new();
//     header.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));

//     let mut form = HashMap::new();

//     let client = Client::builder()
//     .redirect(reqwest::redirect::Policy::limited(20))
//     .cookie_store(true)

//     .default_headers(header)
//     .build().unwrap();

//     match get_schedule(client, &form).await {
//         Ok(r) => {
//             let conn = pool.get().unwrap();
//             for unit in r {
//                 for week in unit.weeks.to_vec() {
//                     for queue in unit.queues.to_vec() {
//                         let r = conn.execute(
//                             r#"
//                             INSERT INTO schedule (
//                                 week, 
//                                 weekday, 
//                                 queue,
//                                 term,
//                                 name
//                             ) VALUES (
//                                 ?1, 
//                                 ?2, 
//                                 ?3,
//                                 ?4,
//                                 ?5
//                             )
//                             ;"#, 
//                             [
//                                 &week.to_string(),
//                                 &unit.weekday.to_string(),
//                                 &queue.to_string(),
//                                 &unit.term.to_string(),
//                                 &unit.name
//                             ]
//                         );

//                         if let Err(e) = r {
//                             println!("{e}");
//                         }
//                     }
//                 }
//             }

            
            
//             for i in 1..=8 {
//                 let stmt = conn.query_row(
//                     r#"
//                     SELECT name
//                     FROM schedule
//                     WHERE week = ?1
//                     AND weekday = ?2
//                     AND term = ?3
//                     AND queue = ?4
//                     LIMIT 1
//                     ;"#,
//                     [
//                         "1",
//                         &Weekday::Tue.to_string(),
//                         "2025",
//                         &i.to_string()
//                     ],
//                     |row| row.get::<_, String>(0),
//                 ).optional().unwrap();

//                 match stmt {
//                     Some(name) => println!("{}", name),
//                     None => println!("[无]")
//                 }

//             }


//         },
//         Err(e) => {
//             panic!("{e}")
//         }
        
//     }
// }
