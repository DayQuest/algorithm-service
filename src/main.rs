use axum::{routing::get, serve, Router};
use env_logger::Builder;
use log::{info, LevelFilter};
use tokio::net::TcpListener;

mod algorythm;
mod endpoint;

const ADDR: &str = "0.0.0.0:8020";

#[tokio::main]
async fn main() {
    Builder::new()
        .filter_level(LevelFilter::Debug)
        .format_target(false)
        .init();

    let listener = TcpListener::bind(ADDR)
        .await
        .expect(format!("Failed to bind to: {ADDR}").as_str());
    info!("Listening on {ADDR}");

    serve(listener, create_router())
        .await
        .expect("Failed to start server");
}

fn create_router() -> Router {
    Router::new().route("/score-vid", get(endpoint::video_score))
}
