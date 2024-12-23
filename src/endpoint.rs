use std::sync::Arc;

use axum::{debug_handler, http::StatusCode, Extension, Json};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error, MySqlPool};

use crate::algorythm::{self, User, Video};

async fn retrieve_video(video_id: &String, db_pool: &MySqlPool) -> Result<Video, Error> {
    let video = query_as::<_, Video>("SELECT * FROM video WHERE uuid = ?")
        .bind(&video_id)
        .fetch_one(db_pool)
        .await?;

    Ok(video)
}

#[derive(Deserialize, Serialize)]
pub struct ScoreVideoResponse {
    score: f64,
}

#[derive(Deserialize, Serialize)]
pub struct ScoreVideoRequest {
    uuid: String,
}

#[debug_handler]
pub async fn video_score(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Json(payload): Json<ScoreVideoRequest>,
) -> Result<Json<ScoreVideoResponse>, StatusCode> {
    match retrieve_video(&payload.uuid, &db_pool).await {
        Ok(video) => {
            let score = algorythm::calc_score(&video);
            info!("Calculated {}'s score: {}", payload.uuid, score);
            Ok(Json(ScoreVideoResponse { score }))
        }
        Err(why) => {
            error!("Error retrieving video data: {}", why);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PersonalizeScoreResponse {
    score: f64,
}

#[derive(Deserialize, Serialize)]
pub struct PersonalizeVideoRequest {
    user_id: String,
    video_id: String,
}

#[debug_handler]
pub async fn personalize_score(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Json(payload): Json<PersonalizeVideoRequest>,
) -> Result<Json<PersonalizeScoreResponse>, StatusCode> {
    match retrieve_video(&payload.video_id, &db_pool).await {
        Ok(video) => match User::from_db(&payload.user_id, &db_pool).await {
            Ok(user) => {
                let score = algorythm::personalize_score(user, &video);
                Ok(Json(PersonalizeScoreResponse { score }))
            },

            Err(why) => {
                error!("Error retrieving user data: {}", why);
                Err(StatusCode::NOT_FOUND)
            }
        },

        Err(why) => {
            error!("Error retrieving video data: {}", why);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NextVideosRequest {
    uuid: String,
}

#[derive(Deserialize, Serialize)]
pub struct NextVideosResponse {
    videos: Vec<String>
}

#[debug_handler]
pub async fn next_videos(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Json(payload): Json<NextVideosRequest>,
) -> Result<Json<NextVideosResponse>, StatusCode> {
        
}
