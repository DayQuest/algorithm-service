use std::time::Instant;

use log::debug;
use serde_json::from_str;
use sqlx::{query, Error, MySqlPool, Row};
use uuid::Uuid;

use crate::config::{
    Config, DB_LIKED_VIDEOS_TABLE, DB_USER_FOLLOWED_USER_TABLE, DB_VIDEO_TABLE,
    FOLLOWED_USERS_COLUMN, USER_ID_COLUMN, UUID_COLUMN, VIDEO_COMMENTS_COLUMN,
    VIDEO_DOWN_VOTES_COLUMN, VIDEO_HASHTAGS_COLUMN, VIDEO_ID_COLUMN, VIDEO_READY_STATUS,
    VIDEO_STATUS_COLUMN, VIDEO_UP_VOTES_COLUMN, VIDEO_VIEWS_COLUMN, VIDEO_VIEWTIME_COLUMN,
};

pub trait DatabaseModel<T> {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<T, Error>;
}

#[derive(Clone)]
pub struct User {
    pub liked_videos: Vec<String>,
    pub following: Vec<String>,
}

impl DatabaseModel<User> for User {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<Self, Error> {
        let rows = query(&format!(
            "
            SELECT {VIDEO_ID_COLUMN} AS id, 'liked_video' AS source
            FROM {DB_LIKED_VIDEOS_TABLE}
            WHERE {USER_ID_COLUMN} = UUID_TO_BIN(?)

            UNION

            SELECT {FOLLOWED_USERS_COLUMN} AS id, 'followed_user' AS source
            FROM {DB_USER_FOLLOWED_USER_TABLE}
            WHERE {USER_ID_COLUMN} = UUID_TO_BIN(?)
            "
        ))
        .bind(uuid)
        .bind(uuid)
        .fetch_all(db_pool)
        .await?;

        let mut liked_videos = Vec::new();
        let mut following = Vec::new();

        for row in rows {
            let id: String = row.try_get("id")?;
            let source: String = row.try_get("source")?;
            match source.as_str() {
                "liked_video" => liked_videos.push(id),
                "followed_user" => following.push(id),
                _ => (),
            }
        }

        Ok(Self {
            liked_videos,
            following,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Video {
    pub uuid: String,
    pub user_id: String,
    pub upvotes: i32,
    pub downvotes: i32,
    pub views: i32,
    pub comments: i32,
    pub hashtags: Vec<String>,
    pub viewtime_seconds: i64,

    //Not saved in the database, a variable to set later
    pub score: f64,
}

impl DatabaseModel<Video> for Video {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<Video, Error> {
        let row = query(&format!(
            "SELECT {USER_ID_COLUMN},
            {VIDEO_HASHTAGS_COLUMN},
            {VIDEO_COMMENTS_COLUMN},
            {VIDEO_UP_VOTES_COLUMN},
            {VIDEO_DOWN_VOTES_COLUMN},
            {VIDEO_VIEWS_COLUMN}, {VIDEO_VIEWTIME_COLUMN} FROM {DB_VIDEO_TABLE} WHERE {UUID_COLUMN} = UUID_TO_BIN(?) AND {VIDEO_STATUS_COLUMN} = ?;"
        ))
        .bind(uuid)
        .bind(VIDEO_READY_STATUS)
        .fetch_one(db_pool)
        .await?;

        let video = Self {
            uuid: uuid.into(),
            user_id: Uuid::from_slice(row.try_get(USER_ID_COLUMN)?)
                .unwrap()
                .to_string(),
            upvotes: row.try_get(VIDEO_UP_VOTES_COLUMN)?,
            downvotes: row.try_get(VIDEO_DOWN_VOTES_COLUMN)?,
            views: row.try_get(VIDEO_VIEWS_COLUMN)?,
            comments: row.try_get(VIDEO_COMMENTS_COLUMN)?,
            viewtime_seconds: row.try_get(VIDEO_VIEWTIME_COLUMN)?,
            hashtags: from_str(row.try_get(VIDEO_HASHTAGS_COLUMN)?).unwrap_or_else(|_| vec![]),
            score: 0.,
        };

        Ok(video)
    }
}

pub async fn fetch_next_videos(
    config: &Config,
    hashtag: String,
    db_pool: &MySqlPool,
) -> Result<(Vec<Video>, Vec<Video>), Error> {
    let start_time = Instant::now();
    
    let videos = sqlx::query(
        "(
            SELECT UUID_COLUMN,
                   USER_ID_COLUMN,
                   VIDEO_HASHTAGS_COLUMN,
                   VIDEO_COMMENTS_COLUMN,
                   VIDEO_UP_VOTES_COLUMN,
                   VIDEO_DOWN_VOTES_COLUMN,
                   VIDEO_VIEWS_COLUMN,
                   VIDEO_VIEWTIME_COLUMN,
                   'random' AS source
            FROM DB_VIDEO_TABLE
            WHERE VIDEO_STATUS_COLUMN = ?
            ORDER BY RAND()
            LIMIT ?
        )
        UNION ALL
        (
            SELECT UUID_COLUMN,
                   USER_ID_COLUMN,
                   VIDEO_HASHTAGS_COLUMN,
                   VIDEO_COMMENTS_COLUMN,
                   VIDEO_UP_VOTES_COLUMN,
                   VIDEO_DOWN_VOTES_COLUMN,
                   VIDEO_VIEWS_COLUMN,
                   VIDEO_VIEWTIME_COLUMN,
                   'hashtag_match' AS source
            FROM DB_VIDEO_TABLE
            WHERE VIDEO_STATUS_COLUMN = ? 
              AND JSON_CONTAINS(VIDEO_HASHTAGS_COLUMN, ?)
            LIMIT ?
        );"
    )
    .bind(VIDEO_READY_STATUS)
    .bind(config.selecting.next_videos_fetch_amount_random)
    .bind(VIDEO_READY_STATUS)
    .bind(hashtag)
    .bind(config.selecting.next_videos_fetch_amount_matching_hashtag)
    .fetch_all(db_pool)
    .await?;
    
    let (random_videos, hashtag_videos): (Vec<Video>, Vec<Video>) = videos
        .iter()
        .fold((Vec::new(), Vec::new()), |(mut random_videos, mut hashtag_videos), row| {
            let video_result = (|| -> Result<Video, Error> {
                Ok(Video {
                    uuid: Uuid::from_slice(row.try_get("UUID_COLUMN")?).unwrap().to_string(),
                    user_id: Uuid::from_slice(row.try_get("USER_ID_COLUMN")?).unwrap().to_string(),
                    upvotes: row.try_get("VIDEO_UP_VOTES_COLUMN")?,
                    downvotes: row.try_get("VIDEO_DOWN_VOTES_COLUMN")?,
                    views: row.try_get("VIDEO_VIEWS_COLUMN")?,
                    comments: row.try_get("VIDEO_COMMENTS_COLUMN")?,
                    viewtime_seconds: row.try_get("VIDEO_VIEWTIME_COLUMN")?,
                    hashtags: from_str(row.try_get("VIDEO_HASHTAGS_COLUMN")?).unwrap_or_else(|_| vec![]),
                    score: 0.,
                })
            })();
    
            match video_result {
                Ok(video) => {
                    match row.try_get::<String, _>("source").unwrap_or_default().as_str() {
                        "random" => random_videos.push(video),
                        "hashtag_match" => hashtag_videos.push(video),
                        _ => (),
                    }
                }
                Err(err) => {
                    eprintln!("Error processing video row: {:?}", err);
                }
            }
    
            (random_videos, hashtag_videos)
        });

    debug!(
        "Fetching videos took: {} ms",
        Instant::elapsed(&start_time).as_millis()
    );
    
    Ok((random_videos, hashtag_videos))
}
