use std::{process::exit, sync::Arc};

use algorithm::Video;
use axum::{routing::post, serve, Extension, Router};
use env_logger::Builder;
use log::{debug, info, LevelFilter};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;

mod algorithm;
mod endpoint;

#[tokio::main]
async fn main() {
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_target(false)
        .init();

    let ip = "0.0.0.0";
    let port = "8020";
    let addr = format!("{}:{}", ip, port);

    
    exit(0);

    let listener = TcpListener::bind(&addr)
        .await
        .expect(format!("Failed to bind to: {addr}").as_str());

    let db_pool = connect_db().await;

    info!("Established connection to database");
    info!("Listening on {addr}");

    serve(listener, app(db_pool))
        .await
        .expect("Failed to start server");
}

fn app(db_pool: MySqlPool) -> Router {
    Router::new()
        .route("/score-vid", post(endpoint::video_score))
        .route("/personalize-score", post(endpoint::personalize_score))
        .route("/next-videos", post(endpoint::next_videos))
        .layer(Extension(Arc::new(db_pool)))
}

async fn connect_db() -> MySqlPool {
    let connection_url = "todo";

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&connection_url)
        .await
        .expect("Failed to establish connection to database")
}
