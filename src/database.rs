use sqlx::{query, Error, MySqlPool, Row};
use uuid::Uuid;

use crate::config::{
    DB_COMMENT_TABLE, DB_LIKED_VIDEOS_TABLE, DB_USER_FOLLOWED_USER_TABLE, DB_VIDEO_TABLE,
    FOLLOWED_USERS_COLUMN, USER_ID_COLUMN, UUID_COLUMN, VIDEO_DOWN_VOTES_COLUMN, VIDEO_ID_COLUMN,
    VIDEO_UP_VOTES_COLUMN, VIDEO_VIEWS_COLUMN, VIDEO_VIEWTIME_COLUMN, VIDEO_STATUS_COLUMN, VIDEO_READY_STATUS
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
        let liked_videos: Vec<String> = query("SELECT ? FROM ? WHERE ? = UUID_TO_BIN(?)")
            .bind(VIDEO_ID_COLUMN)
            .bind(DB_LIKED_VIDEOS_TABLE)
            .bind(USER_ID_COLUMN)
            .bind(uuid)
            .fetch_all(db_pool)
            .await?
            .iter()
            .filter_map(|row| row.try_get::<String, _>(0).ok())
            .collect();

        let following: Vec<String> = query("SELECT ? FROM ? WHERE user_uuid = UUID_TO_BIN(?)")
            .bind(FOLLOWED_USERS_COLUMN)
            .bind(DB_USER_FOLLOWED_USER_TABLE)
            .bind(uuid)
            .fetch_all(db_pool)
            .await?
            .iter()
            .filter_map(|row| row.try_get::<String, _>(0).ok())
            .collect();

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
    pub viewtime_seconds: i64,
    pub score: f64,
}

impl DatabaseModel<Video> for Video {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<Video, Error> {
        let row = query("SELECT ?, ?, ?, ?, ? FROM ? WHERE ? = UUID_TO_BIN(?) AND ? = ?")
            .bind(USER_ID_COLUMN)
            .bind(VIDEO_UP_VOTES_COLUMN)
            .bind(VIDEO_DOWN_VOTES_COLUMN)
            .bind(VIDEO_VIEWS_COLUMN)
            .bind(VIDEO_VIEWTIME_COLUMN)
            .bind(DB_VIDEO_TABLE)
            .bind(UUID_COLUMN)
            .bind(uuid)
            .bind(VIDEO_STATUS_COLUMN)
            .bind(VIDEO_READY_STATUS)
            .fetch_one(db_pool)
            .await?;

        let video = Self {
            uuid: uuid.into(),
            user_id: Uuid::from_slice(row.try_get(USER_ID_COLUMN)?).unwrap().to_string(),
            upvotes: row.try_get(VIDEO_UP_VOTES_COLUMN)?,
            downvotes: row.try_get(VIDEO_DOWN_VOTES_COLUMN)?,
            views: row.try_get(VIDEO_VIEWS_COLUMN)?,
            comments: comment_count(uuid, db_pool).await?,
            viewtime_seconds: row.try_get(VIDEO_VIEWTIME_COLUMN)?,
            score: 0.,
        };

        Ok(video)
    }
}

async fn comment_count(uuid: &str, db_pool: &MySqlPool) -> Result<i32, Error> {
    let count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM ? WHERE ? = UUID_TO_BIN(?)")
        .bind(DB_COMMENT_TABLE)
        .bind(VIDEO_ID_COLUMN)
        .bind(uuid)
        .fetch_one(db_pool)
        .await?;

    Ok(count.0)
}

pub async fn get_random_videos(
    amount: i32,
    with_comment_count: bool,
    db_pool: &MySqlPool,
) -> Result<Vec<Video>, Error> {
    let mut videos = sqlx::query("SELECT ?, ?, ?, ?, ? FROM ? WHERE ? = ? ORDER BY RAND() LIMIT ?")
        .bind(USER_ID_COLUMN)
        .bind(VIDEO_UP_VOTES_COLUMN)
        .bind(VIDEO_DOWN_VOTES_COLUMN)
        .bind(VIDEO_VIEWS_COLUMN)
        .bind(VIDEO_VIEWTIME_COLUMN)
        .bind(DB_VIDEO_TABLE)
        .bind(VIDEO_STATUS_COLUMN)
        .bind(VIDEO_READY_STATUS)
        .bind(amount)
        .fetch_all(db_pool)
        .await?
        .iter()
        .map(|row| {
            Ok(Video {
                uuid: Uuid::from_slice(row.try_get(UUID_COLUMN)?).unwrap().to_string(),
                user_id: row.try_get(USER_ID_COLUMN)?,
                upvotes: row.try_get(VIDEO_UP_VOTES_COLUMN)?,
                downvotes: row.try_get(VIDEO_DOWN_VOTES_COLUMN)?,
                views: row.try_get(VIDEO_VIEWS_COLUMN)?,
                comments: 0,
                viewtime_seconds: row.try_get(VIDEO_VIEWTIME_COLUMN)?,
                score: 0.,   
            })
        })
        .collect::<Result<Vec<Video>, Error>>()?;

    if with_comment_count {
        for video in &mut videos {
            video.comments = comment_count(&video.uuid, db_pool).await?;
        }
    }

    Ok(videos)
}