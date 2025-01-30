use std::time::Instant;

use log::debug;
use serde_json::from_str;
use sqlx::{query, Error, MySqlPool, Row};
use uuid::Uuid;

use crate::config::{
    Config, DB_LIKED_VIDEOS_TABLE, DB_USER_FOLLOWED_USER_TABLE, DB_VIDEO_TABLE,
    FOLLOWED_USERS_COLUMN, USER_ID_COLUMN, UUID_COLUMN, VIDEO_COMMENTS_COLUMN,
    VIDEO_DOWN_VOTES_COLUMN, VIDEO_HASHTAGS_COLUMN, VIDEO_ID_COLUMN, VIDEO_READY_STATUS,
    VIDEO_STATUS_COLUMN, VIDEO_UP_VOTES_COLUMN, VIDEO_VIEWS_COLUMN, VIDEO_VIEWTIME_COLUMN, TIMESTAMP_COLUMN
};

pub trait DatabaseModel<T> {
    async fn from_db(uuid: &str, db_pool: &MySqlPool, config: &Config) -> Result<T, Error>;
}

#[derive(Clone)]
pub struct User {
    pub liked_videos: Vec<String>,
    pub following: Vec<String>,
    pub last_hashtags: Vec<String>, //Last liked hashtag, filtered by timestamp
}

async fn fetch_liked_videos(uuid: &str, db_pool: &MySqlPool) -> Result<Vec<String>, Error> {
    Ok(query(&format!(
        "SELECT {VIDEO_ID_COLUMN} 
         FROM {DB_LIKED_VIDEOS_TABLE}
         WHERE {USER_ID_COLUMN} = UUID_TO_BIN(?)"
    ))
    .bind(uuid)
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .filter_map(|row| row.try_get::<String, _>(VIDEO_ID_COLUMN).ok())
    .collect())
}

async fn fetch_following(uuid: &str, db_pool: &MySqlPool) -> Result<Vec<String>, Error> {
    Ok(query(&format!(
        "SELECT {FOLLOWED_USERS_COLUMN} 
         FROM {DB_USER_FOLLOWED_USER_TABLE}
         WHERE {USER_ID_COLUMN} = UUID_TO_BIN(?)"
    ))
    .bind(uuid)
    .fetch_all(db_pool)
    .await?
    .into_iter()
    .filter_map(|row| row.try_get::<String, _>(FOLLOWED_USERS_COLUMN).ok())
    .collect())
}

async fn fetch_hashtags(uuid: &str, db_pool: &MySqlPool, amount: u32) -> Result<Vec<String>, Error> {
    let rows = query(&format!(
           "SELECT V.{VIDEO_HASHTAGS_COLUMN}
            FROM {DB_LIKED_VIDEOS_TABLE} LV
            JOIN {DB_VIDEO_TABLE} V ON UUID_TO_BIN(LV.{VIDEO_ID_COLUMN}) = V.{UUID_COLUMN}
            WHERE LV.{USER_ID_COLUMN} = UUID_TO_BIN(?)
            ORDER BY LV.{TIMESTAMP_COLUMN} DESC
            LIMIT ?"
       ))
       .bind(uuid)
       .bind(amount)
       .fetch_all(db_pool)
       .await?;

    Ok(rows.into_iter()
        .filter_map(|row| {
            row.try_get::<String, _>(VIDEO_HASHTAGS_COLUMN)
                .ok()
                .map(|hashtags_str| {
                    serde_json::from_str::<Vec<String>>(&hashtags_str).unwrap_or_default()
                })
        })
        .flatten()
        .collect::<Vec<String>>())
}

impl DatabaseModel<User> for User {
    async fn from_db(uuid: &str, db_pool: &MySqlPool, config: &Config) -> Result<Self, Error> {
        let start_time = Instant::now();

        // May return to non-parralel if concurrency too high
        let (liked_videos, following, last_hashtags) = tokio::try_join!(
            fetch_liked_videos(uuid, db_pool),
            fetch_following(uuid, db_pool),
            fetch_hashtags(uuid, db_pool, config.selecting.user_hashtag_fetch_amount)
        )?;

        debug!("Fetching user took: {} ms",start_time.elapsed().as_millis());

        Ok(Self {
            liked_videos,
            following,
            last_hashtags,
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
    async fn from_db(uuid: &str, db_pool: &MySqlPool, _config: &Config) -> Result<Video, Error> {
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

async fn fetch_random_videos(
    config: &Config,
    db_pool: &MySqlPool,
) -> Result<Vec<Video>, Error> {
    let videos = query(&format!(
        "SELECT {UUID_COLUMN},
                {USER_ID_COLUMN},
                {VIDEO_HASHTAGS_COLUMN},
                {VIDEO_COMMENTS_COLUMN},
                {VIDEO_UP_VOTES_COLUMN},
                {VIDEO_DOWN_VOTES_COLUMN},
                {VIDEO_VIEWS_COLUMN},
                {VIDEO_VIEWTIME_COLUMN}
         FROM {DB_VIDEO_TABLE}
         WHERE {VIDEO_STATUS_COLUMN} = ?
         ORDER BY RAND()
         LIMIT ?"
    ))
    .bind(VIDEO_READY_STATUS)
    .bind(config.selecting.next_videos_fetch_amount_random)
    .fetch_all(db_pool)
    .await?;

    process_video_rows(videos)
}

async fn fetch_hashtag_videos(
    config: &Config,
    hashtag: &str,
    db_pool: &MySqlPool,
) -> Result<Vec<Video>, Error> {
    let videos = query(&format!(
        "SELECT {UUID_COLUMN},
                {USER_ID_COLUMN},
                {VIDEO_HASHTAGS_COLUMN},
                {VIDEO_COMMENTS_COLUMN},
                {VIDEO_UP_VOTES_COLUMN},
                {VIDEO_DOWN_VOTES_COLUMN},
                {VIDEO_VIEWS_COLUMN},
                {VIDEO_VIEWTIME_COLUMN}
         FROM {DB_VIDEO_TABLE}
         WHERE {VIDEO_STATUS_COLUMN} = ?
           AND JSON_CONTAINS({VIDEO_HASHTAGS_COLUMN}, ?)
         LIMIT ?"
    ))
    .bind(VIDEO_READY_STATUS)
    .bind(hashtag)
    .bind(config.selecting.next_videos_fetch_amount_matching_hashtag)
    .fetch_all(db_pool)
    .await?;

    process_video_rows(videos)
}

fn process_video_rows(rows: Vec<sqlx::mysql::MySqlRow>) -> Result<Vec<Video>, Error> {
    Ok(rows.into_iter()
        .filter_map(|row| {
            let video_result = Video {
                uuid: Uuid::from_slice(row.try_get("UUID_COLUMN").ok()?).ok()?.to_string(),
                user_id: Uuid::from_slice(row.try_get("USER_ID_COLUMN").ok()?).ok()?.to_string(),
                upvotes: row.try_get("VIDEO_UP_VOTES_COLUMN").ok()?,
                downvotes: row.try_get("VIDEO_DOWN_VOTES_COLUMN").ok()?,
                views: row.try_get("VIDEO_VIEWS_COLUMN").ok()?,
                comments: row.try_get("VIDEO_COMMENTS_COLUMN").ok()?,
                viewtime_seconds: row.try_get("VIDEO_VIEWTIME_COLUMN").ok()?,
                hashtags: from_str(row.try_get("VIDEO_HASHTAGS_COLUMN").ok()?).unwrap_or_default(),
                score: 0.,
            };
            Some(video_result)
        })
        .collect::<Vec<Video>>())
}

pub async fn fetch_next_videos(
    config: &Config,
    hashtag: String,
    db_pool: &MySqlPool,
) -> Result<(Vec<Video>, Vec<Video>), Error> {
    let start_time = Instant::now();

    let (random_videos, hashtag_videos) = tokio::try_join!(
        fetch_random_videos(config, db_pool),
        fetch_hashtag_videos(config, &hashtag, db_pool)
    )?;

    debug!(
        "Fetching videos took: {} ms",
        Instant::elapsed(&start_time).as_millis()
    );

    Ok((random_videos, hashtag_videos))
}
