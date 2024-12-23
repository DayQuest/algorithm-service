use std::sync::Arc;

use axum::{debug_handler, http::StatusCode, Extension, Json};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, MySqlPool};

use crate::algorythm::{self, Video};

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
    match query_as::<_, Video>("SELECT * FROM video WHERE uuid = ?")
        .bind(&payload.uuid)
        .fetch_one(&*db_pool)
        .await
    {
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
