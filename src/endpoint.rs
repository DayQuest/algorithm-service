use std::{sync::{Arc, Mutex}, time::Instant};

use axum::{http::StatusCode, Extension, Json};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::MySqlPool;

use crate::{
    algorithm, auth::Claims, config::{self, Config}, database::{DatabaseModel, User, Video}
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
    Extension(config): Extension<Arc<Mutex<Config>>>,
    Json(payload): Json<ScoreVideoRequest>,
) -> Result<Json<ScoreVideoResponse>, StatusCode> {
    let config = config.lock().unwrap().clone();
    match Video::from_db(&payload.uuid, &db_pool, &config).await {
        Ok(video) => {
            let score = algorithm::score_video(&video, &config);
            Ok(Json(ScoreVideoResponse { score }))
        }
        Err(why) => {
            warn!("Error retrieving video data (maybe video id 404): {} ", why);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PersonalizeScoreResponse {
    score: f64,
    
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalizeVideoRequest {
    video_id: String,
}

//#[debug_handler]
pub async fn score_video_personalized(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Extension(config): Extension<Arc<Mutex<Config>>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<PersonalizeVideoRequest>,
) -> Result<Json<PersonalizeScoreResponse>, StatusCode> {
    let config = config.lock().unwrap().clone();
    match Video::from_db(&payload.video_id, &db_pool, &config).await {
        Ok(video) => match User::from_db(&claims.user_id, &db_pool, &config).await {
            Ok(user) => {
                let score =
                    algorithm::score_video_personalized(&user, &video, &config);
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
pub struct NextVideosResponse {
    videos: Vec<String>, //Vec of UUIDs of videos
}

//#[debug_handler]
pub async fn next_videos(
    Extension(db_pool): Extension<Arc<MySqlPool>>,
    Extension(config): Extension<Arc<Mutex<Config>>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<NextVideosResponse>, StatusCode> {
    let start_time = Instant::now();
    let config = config.lock().unwrap().clone();
    let user: User = User::from_db(&claims.user_id, &db_pool, &config)
        .await
        .map_err(|why| {
            warn!("Fetching user failed: {why}");
            StatusCode::NOT_FOUND
        })?;
    
    let videos = algorithm::next_videos(&user, &config, &db_pool)
        .await
        .map_err(|why| {
            error!("Next Videos Algorithm failed: {why}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .iter()
        .map(|video| video.uuid.clone())
        .collect::<Vec<String>>();

    debug!("Processing next videos request took: {} ms", start_time.elapsed().as_millis());
    Ok(Json(NextVideosResponse { videos }))
}

//#[debug_handler]
pub async fn get_config(
    Extension(config): Extension<Arc<Mutex<Config>>>,
) -> Result<Json<Config>, StatusCode> {
    Ok(Json((*config.lock().unwrap()).clone()))
}

//#[debug_handler]
pub async fn set_config(
    Extension(config): Extension<Arc<Mutex<Config>>>,
    Json(payload): Json<Config>,
) -> Result<StatusCode, StatusCode> {
    info!("Config update was requested");

    if let Err(why) = config::validate(&payload) {
        warn!("Config set request denied. Validation failed: {why}");
        return Ok(StatusCode::NOT_ACCEPTABLE);
    }

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
