use axum::{routing::get, Router};
use clokwerk::{AsyncScheduler, TimeUnits};
use news::{
    routes::{create_feed, get_feed, root},
    Database,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("Server started at port 3000");
    let mut scheduler: AsyncScheduler = AsyncScheduler::new();
    scheduler.every(60.minutes()).run(|| async {
        let _ = reqwest::get("http://localhost:3000/news/create").await;
    });

    tokio::spawn(async move {
        loop {
            scheduler.run_pending().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let db = Arc::new(Mutex::new(Database::new()));
    let app = Router::new()
        .route("/", get(root))
        .route("/news", get(get_feed))
        .route("/news/create", get(create_feed))
        .with_state(db.clone());
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
