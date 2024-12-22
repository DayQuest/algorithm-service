use axum::Json;
use log::info;
use serde::{Deserialize, Serialize};

use crate::algorythm::{self, Video};

#[derive(Deserialize, Serialize)]
pub struct VideoScoreResponse {
    score: f64,
}


pub async fn video_score(Json(payload): Json<Video>) -> Json<VideoScoreResponse> {
    let score = algorythm::calc_score(&payload);

    info!("Calculated {}'s score: {}", payload.uuid, score);
    Json(VideoScoreResponse { score })
}
