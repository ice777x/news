use std::sync::{Arc, Mutex};

use crate::models::Feed;
use crate::parser::get_all_rss;
use crate::Database;
use axum::extract::{Json, Query, State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RootStruct {
    pub message: String,
    pub path: Vec<Path>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Path {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

pub async fn root() -> Json<RootStruct> {
    let paths = vec![
        Path {
            name: "/".to_string(),
        },
        Path {
            name: "/news".to_string(),
        },
    ];
    let resp = RootStruct {
        message: String::from("Rss Feed Parser for turkish news feed"),
        path: paths,
    };
    Json(resp)
}

pub async fn get_feed(
    State(db): State<Arc<Mutex<Database>>>,
    mut info: Query<Info>,
) -> Json<Vec<Feed>> {
    if info.limit.is_none() {
        info.limit = Some(10);
    }
    if info.q.is_none() {
        let feeds = db.lock().unwrap().get_all_feeds(info.limit.unwrap());
        return Json(feeds);
    }
    let feeds = db
        .lock()
        .unwrap()
        .get_feeds_by_query(info.q.as_ref().unwrap().to_string(), info.limit.unwrap());
    Json(feeds)
}

pub async fn create_feed(State(db): State<Arc<Mutex<Database>>>) -> Json<Vec<Feed>> {
    let feed_vec = get_all_rss().await;
    let created_feeds = db.lock().unwrap().create_feed_many(feed_vec);
    match created_feeds {
        Some(feeds) => {
            println!("Feed Created");
            Json(feeds)
        }
        None => Json(vec![]),
    }
}
