use std::{
    env,
    process::exit,
    sync::{Arc, Mutex},
};

use algorithm::score_video;
use axum::{
    middleware,
    routing::{get, post},
    serve, Extension, Router,
};
use colored::Colorize;
use config::{Config, DATABASE_CONN_URL_KEY};
use database::Video;
use env_logger::Builder;
use log::{debug, info, LevelFilter};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;

mod algorithm;
mod auth;
mod config;
mod database;
mod endpoint;

#[tokio::main]
async fn main() {
    ctrlc::set_handler(move || {
        info!("{}", "Stopping server, Bye :)".on_red());
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_target(false)
        .init();
    info!("Starting..");

    let config = config::load();
    config::validate(&config).expect("Config validation failed");

    let ip = "0.0.0.0";
    let port = "8020";
    let addr = format!("{}:{}", ip, port);

    test_video_score(&config);
    let listener = TcpListener::bind(&addr)
        .await
        .expect(format!("Failed to bind to: {addr}").as_str());

    let db_pool = connect_db().await;

    info!("Listening on {addr}");

    serve(listener, app(config, Some(db_pool)))
        .await
        .expect("Failed to start server");
}

fn app(config: Config, db_pool: Option<MySqlPool>) -> Router {
    let jwt_router = Router::new()
        .route("/scoreVideo", post(endpoint::score_video))
        .route(
            "/scoreVideoPersonalized",
            post(endpoint::score_video_personalized),
        )
        .route("/nextVideos", post(endpoint::next_videos))
        .layer(middleware::from_fn(auth::jwt_middleware));

    let internal_router = Router::new()
        .route("/getConfig", get(endpoint::get_config))
        .route("/setConfig", post(endpoint::set_config))
        .layer(middleware::from_fn(auth::internal_secret_middleware));

    let mut final_router =
        Router::merge(jwt_router, internal_router).layer(Extension(Arc::new(Mutex::new(config))));

    // Gives option to not use a database for testing single functions
    if let Some(pool) = db_pool {
        final_router = final_router.layer(Extension(Arc::new(pool)));
    } else {
        debug!("Not connecting to database!")
    }

    final_router
}

async fn connect_db() -> MySqlPool {
    let connection_url = env::var(DATABASE_CONN_URL_KEY)
        .expect("Failed to get database connection url out of environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&connection_url)
        .await
        .expect("Failed to connect to database");

    info!("Established connection to database");
    pool
}

fn test_video_score(config: &Config) {
    debug!("Video Score Testing:");
    debug!(
        "    - {}",
        score_video(
            &Video {
                uuid: "".into(),
                user_id: "".into(),
                upvotes: 12,
                downvotes: 1,
                views: 111,
                comments: 2,
                viewtime_seconds: 220,
                score: 0.
            },
            &config
        )
    );
    debug!(
        "    - {}",
        score_video(
            &Video {
                uuid: "".into(),
                user_id: "".into(),
                upvotes: 0,
                downvotes: 1,
                views: 2,
                comments: 0,
                viewtime_seconds: 4,
                score: 0.
            },
            &config
        )
    );
    debug!(
        "    - {}",
        score_video(
            &Video {
                uuid: "".into(),
                user_id: "".into(),
                upvotes: 20_000,
                downvotes: 180,
                views: 1_000_000,
                comments: 30_000,
                viewtime_seconds: 2_000_000,
                score: 0.
            },
            &config
        )
    );
    debug!(
        "    - {}",
        score_video(
            &Video {
                uuid: "".into(),
                user_id: "".into(),
                upvotes: 93_000,
                downvotes: 350,
                views: 1_800_000,
                comments: 1785,
                viewtime_seconds: 2_700_000,
                score: 0.
            },
            &config
        )
    );
}
