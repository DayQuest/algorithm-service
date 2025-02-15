use std::{
    env, process::exit, sync::{Arc, Mutex}, time::Instant
};

use axum::{
    middleware, routing::{get, post}, serve, Extension, Router
};
use colored::Colorize;
use config::{Config, DATABASE_CONN_URL_KEY, HOST_IP_KEY, HOST_PORT_KEY};
use dotenv::dotenv;
use env_logger::{Builder, Env};
use log::{debug, error, info};
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

    // Handle Ctrl-C gracefully
    ctrlc::set_handler(move || {
        info!("{}", "Stopping server, Bye :)".on_red());
        exit(0);
    })
    .unwrap_or_else(|e| {
        error!("Error setting Ctrl-C handler: {}", e);
        exit(0);
    });

    // Initialize logger
    let dotenv_res = dotenv();
    Builder::from_env(Env::default())
        .format_target(false)
        .init();
    info!(
        "Starting ({})..",
        if cfg!(debug_assertions) {
            "Debug Mode"
        } else {
            "Release Mode"
        }
    );

    if dotenv_res.is_ok() {
        info!("Loaded .env file {}", "(development only)".yellow());
    }

    debug!(
        "JWT: {}",
        auth::gen_token("testing".to_string()).unwrap_or_else(|e| {
            error!("Failed to generate test JWT: {}", e);
            "Invalid Token".to_string()
        })
    );

    // Load and validate configuration
    let config = config::load();
    if let Err(e) = config::validate(&config) {
        error!("Config validation failed: {}", e);
        exit(0);
    }

    // Get host and port from environment variables
    let ip = env::var(HOST_IP_KEY).unwrap_or_else(|_| {
        error!("Did not find host IP in env");
        exit(0);
    });

    let port = env::var(HOST_PORT_KEY).unwrap_or_else(|_| {
        error!("Did not find host port in env");
        exit(0);
    });

    let addr = format!("{}:{}", ip, port);

    // Bind to address
    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        error!("Failed to bind to {}: {}", addr, e);
        exit(0);
    });

    // Connect to database
    let db_pool = match connect_db(&config).await {
        Ok(pool) => pool,
        Err(why) => {
            error!("Failed to connect or testquery to database: {}", why);
            exit(0);
        }
    };

    info!(
        "Done, listening on {addr}, ({} ms)",
        Instant::elapsed(&start_time).as_millis()
    );

    serve(listener, app(config, db_pool))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to start server: {}", e);
            exit(0);
        });
}


fn app(config: Config, db_pool: MySqlPool) -> Router {
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

    let final_router =
        Router::merge(jwt_router, internal_router)
        .layer(Extension(Arc::new(Mutex::new(config))))
        .layer(Extension(Arc::new(db_pool)));
        

    final_router
}

async fn connect_db(config: &Config) -> Result<MySqlPool, sqlx::Error> {
    let connection_url = env::var(DATABASE_CONN_URL_KEY).unwrap_or_else(|_| {
        error!("Failed to get database connection URL from environment");
        exit(0);
    });

    let pool = MySqlPoolOptions::new()
        .max_connections(config.max_dbpool_connections)
        .connect(&connection_url)
        .await?;

    let row: (String,) = sqlx::query_as("SELECT DATABASE()").fetch_one(&pool).await?;

    info!("Established connection to database: {}", row.0);
    Ok(pool)
}
