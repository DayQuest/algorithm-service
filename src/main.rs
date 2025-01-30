use std::{
    env,
    process::exit,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    middleware,
    routing::{get, post},
    serve, Extension, Router,
};
use colored::Colorize;
use config::{Config, DATABASE_CONN_URL_KEY, HOST_IP_KEY, HOST_PORT_KEY, LOG_LEVEL_KEY};
use dotenv::dotenv;
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
    let start_time = Instant::now();
    ctrlc::set_handler(move || {
        info!("{}", "Stopping server, Bye :)".on_red());
        exit(0);
    })
    .expect("Error setting Ctrl-C handler");

   let dotenv_res = dotenv();
   setup_logger();
   info!("Starting..");
    if let Ok(_) = dotenv_res {
        info!("Loaded .env file {}", "(development only)".yellow())
    }
    
    debug!("JWT: {}", auth::gen_token("testing".to_string()).unwrap());
    let config = config::load();
    config::validate(&config).expect("Config validation failed");

    let ip = env::var(HOST_IP_KEY).expect("Did not find host ip in env");
    let port = env::var(HOST_PORT_KEY).expect("Did not find host port in env");
    let addr = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(&addr)
        .await
        .expect(format!("Failed to bind to: {addr}").as_str());

    let db_pool = connect_db(&config).await;

    info!(
        "Done, listening on {addr}, ({} ms)",
        Instant::elapsed(&start_time).as_millis()
    );

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

fn setup_logger() {
    let log_level = env::var(LOG_LEVEL_KEY).expect("Failed to get log level out of enviroment");
    let log_level = match log_level.as_str() {
        "INFO" => LevelFilter::Info,
        "DEBUG" => LevelFilter::Debug,
        "ERROR" => LevelFilter::Error,
        "WARN" => LevelFilter::Warn,
        "TRACE" => LevelFilter::Trace,
        "OFF" => LevelFilter::Off,
        _ => { panic!("Unknown log level: {log_level}") },
    };

    Builder::new()
        .filter_level(log_level)
        .format_target(false)
        .init();
}

async fn connect_db(config: &Config) -> MySqlPool {
    let connection_url = env::var(DATABASE_CONN_URL_KEY)
        .expect("Failed to get database connection url out of environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_dbpool_connections)
        .connect(&connection_url)
        .await
        .expect("Failed to connect to database");

    info!("Established connection to database");
    pool
}
