use sqlx::{query, query_scalar, Error, MySqlPool, Row};

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
        let liked_videos: Vec<String> =
            query_scalar("SELECT * FROM liked_videos WHERE user_id = ?")
                .bind(uuid)
                .fetch_all(db_pool)
                .await?;

        let following: Vec<String> =
            query_scalar("SELECT * FROM user_followed_users WHERE user_uuid = ?")
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
}

const VIDEO_ROW_SELECTIONS: &str = "user_id, up_votes, down_votes, views, viewtime_seconds";
impl DatabaseModel<Video> for Video {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<Video, Error> {
        let video_row = query(&format!(
            "SELECT {VIDEO_ROW_SELECTIONS} FROM video WHERE uuid = ?;"
        ))
        .bind(uuid)
        .fetch_one(db_pool)
        .await?;

        let video = Self {
            uuid: uuid.into(),
            user_id: video_row.try_get("user_id")?,
            upvotes: video_row.try_get("up_votes")?,
            downvotes: video_row.try_get("down_votes")?,
            views: video_row.try_get("views")?,
            comments: comment_count(uuid.into(), db_pool).await?,
            viewtime_seconds: video_row.try_get("viewtime_seconds")?,
        };

        Ok(video)
    }
}

async fn comment_count(uuid: String, db_pool: &MySqlPool) -> Result<u32, Error> {
    Ok(query("SELECT * FROM comment WHERE video_id = ?;")
        .bind(uuid)
        .fetch_all(db_pool)
        .await?
        .len() as u32)
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
        "SELECT {VIDEO_ROW_SELECTIONS} FROM video ORDER BY RAND() LIMIT ?;"
    ))
    .bind(amount)
    .fetch_all(db_pool)
    .await?
    .iter()
    .map(|row| {
        Ok(Video {
            uuid: row.try_get("uuid")?,
            user_id: row.try_get("user_id")?,
            upvotes: row.try_get("up_votes")?,
            downvotes: row.try_get("down_votes")?,
            views: row.try_get("views")?,
            comments: 0,
            viewtime_seconds: row.try_get("viewtime_seconds")?,
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
