
use std::fmt::format;

use axum::{routing::post, serve, Router};
use env_logger::Builder;
use log::{info, LevelFilter};
use tokio::net::TcpListener;

mod algorythm;
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

    let listener = TcpListener::bind(&addr)
        .await
        .expect(format!("Failed to bind to: {addr}").as_str());
    info!("Listening on {addr}");

    serve(listener, create_router())
        .await
        .expect("Failed to start server");
}

fn create_router() -> Router {
    Router::new().route("/score-vid", post(endpoint::video_score))
}
