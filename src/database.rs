use sqlx::{query, query_scalar, Error, MySqlPool, Row};

use crate::config::{
    DB_COMMENT_TABLE, DB_LIKED_VIDEOS_TABLE, DB_USER_FOLLOWED_USER_TABLE, DB_VIDEO_TABLE,
    FOLLOWED_USERS_COLUMN, USER_ID_COLUMN, UUID_COLUMN, VIDEO_DOWN_VOTES_COLUMN, VIDEO_ID_COLUMN,
    VIDEO_UP_VOTES_COLUMN, VIDEO_VIEWS_COLUMN, VIDEO_VIEWTIME_COLUMN,
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
        let liked_videos: Vec<String> = query_scalar(&format!(
            "SELECT {VIDEO_ID_COLUMN} FROM {DB_LIKED_VIDEOS_TABLE} WHERE {USER_ID_COLUMN} = ?"
        ))
        .bind(uuid)
        .fetch_all(db_pool)
        .await?;

        //user_uuid may be changed to user_id
        let following: Vec<String> = query_scalar(&format!(
            "SELECT {FOLLOWED_USERS_COLUMN} FROM {DB_USER_FOLLOWED_USER_TABLE} WHERE user_uuid = ?"
        ))
        .bind(uuid)
        .fetch_all(db_pool)
        .await?;

        Ok(Self {
            liked_videos,
            following,
        })
    }
}

#[derive(Clone)]
pub struct Video {
    pub uuid: String,
    pub user_id: String,
    pub upvotes: u32,
    pub downvotes: u32,
    pub views: u32,
    pub comments: u32,
    pub viewtime_seconds: u64,

    //Not saved in the database, a variable to set later
    pub score: f64,
}

impl DatabaseModel<Video> for Video {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<Video, Error> {
        let row = query(&format!(
            "SELECT {USER_ID_COLUMN}, 
            {VIDEO_UP_VOTES_COLUMN}, 
            {VIDEO_DOWN_VOTES_COLUMN}, 
            {VIDEO_VIEWS_COLUMN}, 
            {VIDEO_VIEWTIME_COLUMN} FROM {DB_VIDEO_TABLE} WHERE {UUID_COLUMN} = ?;"
        ))
        .bind(uuid)
        .fetch_one(db_pool)
        .await?;

        let video = Self {
            uuid: uuid.into(),
            user_id: row.try_get(USER_ID_COLUMN)?,
            upvotes: row.try_get(VIDEO_UP_VOTES_COLUMN)?,
            downvotes: row.try_get(VIDEO_DOWN_VOTES_COLUMN)?,
            views: row.try_get(VIDEO_VIEWS_COLUMN)?,
            comments: comment_count(uuid.into(), db_pool).await?,
            viewtime_seconds: row.try_get(VIDEO_VIEWTIME_COLUMN)?,
            score: 0.,
        };

        Ok(video)
    }
}

async fn comment_count(uuid: String, db_pool: &MySqlPool) -> Result<u32, Error> {
    let count: (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM {DB_COMMENT_TABLE} WHERE {VIDEO_ID_COLUMN} = ?;"
    ))
    .bind(uuid)
    .fetch_one(db_pool)
    .await?;

    Ok(count.0 as u32)
}

/// Fetches {amount} of videos randomly from the database using 1 query only.
/// if with_comment_count is activated it will move trough the fetched vec and get each
/// comment count with a query for that video and writes that to the mutable video vec. This operation
/// costs a bit because it has to send an extra query for every video. If set on false only one query
/// for everything is needed. So only enable when really needed!
pub async fn get_random_videos(
    amount: u32,
    with_comment_count: bool,
    db_pool: &MySqlPool,
) -> Result<Vec<Video>, Error> {
    let mut videos = sqlx::query(&format!(
        "SELECT {USER_ID_COLUMN}, 
            {VIDEO_UP_VOTES_COLUMN}, 
            {VIDEO_DOWN_VOTES_COLUMN}, 
            {VIDEO_VIEWS_COLUMN}, 
            {VIDEO_VIEWTIME_COLUMN} FROM {DB_VIDEO_TABLE} ORDER BY RAND() LIMIT ?;"
    ))
    .bind(amount)
    .fetch_all(db_pool)
    .await?
    .iter()
    .map(|row| {
        Ok(Video {
            uuid: row.try_get(UUID_COLUMN)?,
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
            video.comments = comment_count(video.uuid.clone(), db_pool).await?;
        }
    }

    Ok(videos)
}
