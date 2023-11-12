pub mod models;
pub mod parser;
pub mod routes;
pub mod schema;
use self::feeds::dsl::*;
use self::models::{Feed, NewFeed};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{prelude::*, r2d2};
use dotenv::dotenv;
use regex::Regex;
use schema::feeds;
use serde::{Deserialize, Serialize};
use std::{env, fs, sync::Arc};

fn news_reader_from_file(path: &str) -> Vec<String> {
    let read_dir = fs::read_to_string(path).unwrap();
    read_dir.lines().map(|f| f.to_string()).collect()
}

pub type DBPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pool: Arc<DBPool>,
}

impl Database {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Arc::new(
            r2d2::Pool::builder()
                .build(manager)
                .expect("Failed to create Pool"),
        );
        Database { pool }
    }

    pub fn select_feed_by_id(&mut self, id_key: i32, limit: u32) -> Vec<Feed> {
        let feed = feeds
            .filter(id.eq(id_key))
            .limit(limit.into())
            .select(Feed::as_select())
            .load(&mut self.pool.get().unwrap());
        match feed {
            Ok(feed) => feed,
            Err(_) => {
                println!("Cannot find any feed for {}", id_key);
                vec![]
            }
        }
    }
    pub fn create_feed_many(&mut self, feed_vec: Vec<RssFeed>) -> Option<Vec<Feed>> {
        let all_feed = self.get_all_feeds(9999999);

        let mut new_feeds: Vec<NewFeed> = vec![];

        for feed in feed_vec.iter() {
            if feed.title == String::new() {
                continue;
            }

            if all_feed
                .iter()
                .find(move |p| p.link.as_deref().unwrap() == feed.link)
                .is_some()
            {
                continue;
            }
            let new_post = NewFeed {
                title: feed.title.to_string(),
                link: Some(feed.link.to_string()),
                description: feed.description(),
                content: feed.content(),
                author: Some(feed.author.to_string()),
                image: feed.clone().images,
                published: match feed.clone().published {
                    Some(date) => Some(RssFeed::to_date(&date)),
                    None => None,
                },
            };
            new_feeds.push(new_post);
        }
        if new_feeds.len() == 0 {
            return None;
        }
        let feeds_len = &new_feeds.len() / 8;
        for _ in 0..8 {
            if feeds_len > new_feeds.len() {
                println!("Last Ones");
                let _ = diesel::insert_into(feeds::table)
                    .values(&new_feeds)
                    .execute(&mut self.pool.get().unwrap());
            } else if new_feeds.len() == 0 {
                return Some(vec![]);
            } else {
                let splitted_new_feed = new_feeds.split_off(feeds_len);
                let _ = diesel::insert_into(feeds::table)
                    .values(&new_feeds)
                    .execute(&mut self.pool.get().unwrap());
                new_feeds = splitted_new_feed;
            }
        }
        Some(vec![])
    }

    pub fn get_feeds_by_query(&mut self, query: String, limit: u32) -> Vec<Feed> {
        let res = feeds
            .filter(
                feeds::title
                    .ilike(format!("%{}%", query))
                    .or(feeds::description.ilike(format!("%{}%", query))),
            )
            .select(Feed::as_select())
            .limit(limit.into())
            .order(feeds::id.desc())
            .get_results(&mut self.pool.get().unwrap());
        match res {
            Ok(res) => res,
            Err(_) => {
                println!("Error to fetch feeds.");
                vec![]
            }
        }
    }

    pub fn get_all_feeds(&mut self, limit: u32) -> Vec<Feed> {
        let all_feeds = feeds
            .select(Feed::as_select())
            .limit(limit.into())
            .order(feeds::published.desc().nulls_last())
            .get_results(&mut self.pool.get().unwrap());
        match all_feeds {
            Ok(res) => res,
            Err(_) => {
                println!("Error to fetch feeds.");
                vec![]
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Image {
    pub url: Option<String>,
    pub length: String,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct RssFeed {
    pub title: String,
    pub link: String,
    pub description: Option<String>,
    pub author: String,
    pub guid: String,
    pub images: Option<String>,
    pub published: Option<String>,
    pub content: Option<String>,
}

impl RssFeed {
    fn to_date(date: &str) -> NaiveDateTime {
        let new_date = NaiveDateTime::parse_and_remainder(date, "%Y-%m-%d %H:%M:%S").ok();
        // let tz = FixedOffset::east_opt(3 * 3600).unwrap();
        match new_date {
            Some((date, _)) => date,
            None => {
                NaiveDateTime::parse_and_remainder(date, "%a, %d %b %Y %H:%M:%S %z")
                    .unwrap()
                    .0
            }
        }
    }

    #[allow(dead_code)]
    fn image_from_source(&self) -> String {
        let re = Regex::new(r#"src="(?<link>.+)""#).unwrap();
        let desc = self.description();
        match desc {
            Some(desc) => match re.captures(&desc).ok_or(String::new()) {
                Ok(caps) => caps.name("link").unwrap().as_str().to_string(),
                Err(e) => e,
            },
            None => {
                let cont = self.content();
                match cont {
                    Some(text) => match re.captures(&text).ok_or(String::new()) {
                        Ok(caps) => caps
                            .name("link")
                            .ok_or(String::new())
                            .unwrap()
                            .as_str()
                            .to_string(),
                        Err(e) => e,
                    },
                    None => String::new(),
                }
            }
        }
    }

    pub fn description(&self) -> Option<String> {
        let re = Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
        match &self.description {
            Some(desc) => Some(
                re.replace_all(&desc, "")
                    .to_string()
                    .replace("&#039;", "'")
                    .replace("&#8216;", "'")
                    .replace("&#8217;", "'")
                    .replace("&#46;", ".")
                    .replace("&amp;", "&")
                    .replace("&quot;", "\"")
                    .replace("\n\n", "\n")
                    .replace("(adsbygoogle = window.adsbygoogle || []).push({});", "")
                    .replace("&#8220;", "\""),
            ),
            None => None,
        }
    }

    pub fn content(&self) -> Option<String> {
        let re = Regex::new(r#"<(?:"[^"]*"['"]*|'[^']*'['"]*|[^'">])+>"#).unwrap();
        match self.content.clone() {
            Some(cont) => Some(
                re.replace_all(&cont, "")
                    .to_string()
                    .replace("&#039;", "'")
                    .replace("&#8216;", "'")
                    .replace("&#8217;", "'")
                    .replace("&#46;", ".")
                    .replace("&amp;", "&")
                    .replace("&quot;", "\"")
                    .replace("&#8220;", "\"")
                    .replace("\n\n", "\n")
                    .replace("(adsbygoogle = window.adsbygoogle || []).push({});", ""),
            ),
            None => None,
        }
    }
}
