use std::{process::exit, sync::{Arc, Mutex}, thread, time::Duration};

use algorithm::score_video;
use axum::{routing::{get, post}, serve, Extension, Router};
use config::Config;
use env_logger::Builder;
use log::{debug, info, LevelFilter};
use model::Video;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;

mod algorithm;
mod endpoint;
mod config;
mod model;

#[tokio::main]
async fn main() {
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_target(false)
        .init();

    let config = config::load();
   
    let ip = "0.0.0.0";
    let port = "8020";
    let addr = format!("{}:{}", ip, port);


    debug!("{}", score_video(&Video { uuid: "".into(), user_id: "".into(), upvotes: 12, downvotes: 1, views: 111, comments: 2, viewtime_seconds: 220 }, &config));
    debug!("{}", score_video(&Video { uuid: "".into(), user_id: "".into(), upvotes: 0, downvotes: 0, views: 2, comments: 0, viewtime_seconds: 4 }, &config));
    debug!("{}", score_video(&Video { uuid: "".into(), user_id: "".into(), upvotes: 20_000, downvotes: 180, views: 1_000_000, comments: 30_000, viewtime_seconds: 2_000_000 },&config));
    debug!("{}", score_video(&Video { uuid: "".into(), user_id: "".into(), upvotes: 93_000, downvotes: 350, views: 1_800_000, comments: 1785, viewtime_seconds: 2_700_000 }, &config));


    let listener = TcpListener::bind(&addr)
        .await
        .expect(format!("Failed to bind to: {addr}").as_str());

    //let db_pool = connect_db().await;

    //info!("Established connection to database");
    info!("Listening on {addr}");

    serve(listener, app(config))
        .await
        .expect("Failed to start server");
}

fn app(config: Config) -> Router {
    Router::new()
        .route("/scoreVid", post(endpoint::video_score))
        .route("/personalizeScore", post(endpoint::personalize_score))
        .route("/nextVideos", post(endpoint::next_videos))
        .route("/getConfig", get(endpoint::get_config))
        .route("/setConfig", post(endpoint::set_config))
       // .layer(Extension(Arc::new(db_pool)))
        .layer(Extension(Arc::new(Mutex::new(config))))
}

async fn connect_db() -> MySqlPool {
    let connection_url = "todo";

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&connection_url)
        .await
        .expect("Failed to establish connection to database")
}
