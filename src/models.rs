use crate::schema::feeds;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, PartialEq, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::feeds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub image: Option<String>,
    pub content: Option<String>,
    pub published: Option<String>,
}

#[derive(Debug, Insertable, Clone)]
#[diesel(table_name = feeds)]
pub struct NewFeed {
    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub image: Option<String>,
    pub content: Option<String>,
    pub published: Option<String>,
}
