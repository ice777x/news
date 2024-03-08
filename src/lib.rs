pub mod models;
pub mod parser;
pub mod routes;
pub mod schema;
pub mod structs;
use std::fs;
use structs::*;

fn news_reader_from_file(path: &str) -> Vec<String> {
    let read_dir = fs::read_to_string(path).unwrap();
    read_dir.lines().map(|f| f.to_string()).collect()
}
