use std::sync::{Arc, Mutex};

use axum::{debug_handler, http::StatusCode, Extension, Json};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    algorithm,
    config::{self, Config},
    database::{DatabaseModel, User, Video},
};

#[derive(Deserialize, Serialize)]
pub struct ScoreVideoResponse {
    score: f64,
}

#[derive(Deserialize, Serialize)]
pub struct ScoreVideoRequest {
    uuid: String,
}

//#[debug_handler]
pub async fn score_video(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Extension(config): Extension<Arc<Config>>,
    Json(payload): Json<ScoreVideoRequest>,
) -> Result<Json<ScoreVideoResponse>, StatusCode> {
    match Video::from_db(&payload.uuid, &db_pool).await {
        Ok(video) => {
            let score = algorithm::score_video(&video, &config);
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

//#[debug_handler]
pub async fn score_video_personalized(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Extension(config): Extension<Arc<Config>>,
    Json(payload): Json<PersonalizeVideoRequest>,
) -> Result<Json<PersonalizeScoreResponse>, StatusCode> {
    match Video::from_db(&payload.video_id, &db_pool).await {
        Ok(video) => match User::from_db(&payload.user_id, &db_pool).await {
            Ok(user) => {
                let score = algorithm::score_video_personalized(&user, &video, &config);
                Ok(Json(PersonalizeScoreResponse { score }))
            }

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
    user_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct NextVideosResponse {
    videos: Vec<String>, //Vec of UUIDs of videos
}

//#[debug_handler]
pub async fn next_videos(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Extension(config): Extension<Arc<Mutex<Config>>>,
    Json(payload): Json<NextVideosRequest>,
) -> Result<Json<NextVideosResponse>, StatusCode> {
    let config = config.lock().unwrap().clone();
    let user = User::from_db(&payload.user_id, &db_pool)
        .await
        .or_else(|_| Err(StatusCode::NOT_FOUND))?;

    let videos = algorithm::next_videos(&user, &config, &db_pool)
        .await
        .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .iter()
        .map(|video| video.uuid.clone())
        .collect::<Vec<String>>();

    Ok(Json(NextVideosResponse { videos }))
}

//#[debug_handler]
pub async fn get_config(
    Extension(config): Extension<Arc<Mutex<Config>>>,
) -> Result<Json<Config>, StatusCode> {
    Ok(Json((*config.lock().unwrap()).clone()))
}

#[derive(Deserialize, Serialize)]
pub struct SetConfigRequest {
    uuid: String,
}

//#[debug_handler]
pub async fn set_config(
    Extension(config): Extension<Arc<Mutex<Config>>>,
    Json(payload): Json<Config>,
) -> Result<StatusCode, StatusCode> {
    info!("Config update was requested");

    match serde_json::to_string_pretty(&json!(payload)) {
        Ok(json) => {
            if let Err(why) = config::overwrite(json) {
                error!("Failed to overwrite config content: {:?}", why);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Err(why) => {
            error!("Failed to convert payload into pretty json string: {}", why);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    *config.lock().unwrap() = payload;
    info!("Updated config!");
    Ok(StatusCode::OK)
}
