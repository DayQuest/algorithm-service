use algorythm::VideoData;
use axum::{response::IntoResponse, routing::get, serve, Json, Router};
use serde::Deserialize;
use tokio::net::TcpListener;

mod algorythm;

const ADDR: &str = "0.0.0.0:8020";

#[tokio::main]
async fn main() {
    const VIRAL_SCORE: f64 = 10_000.;

    //Start
    println!("Starting on {ADDR}");
    let listener = TcpListener::bind(ADDR)
        .await
        .expect(format!("Failed to bind to: {ADDR}").as_str());

    serve(listener, create_router())
        .await
        .expect("Failed to start server");
}

fn create_router() -> Router {
    Router::new().route("/score-vid", get(video_score))
}

async fn video_score(Json(payload): Json<VideoData>) -> impl IntoResponse {
    println!("DEBUG: Got video score request");
}