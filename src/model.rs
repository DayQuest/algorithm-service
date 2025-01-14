use sqlx::{
    query, query_scalar, Error, MySqlPool, Row,
};

pub trait DatabaseModel<T> {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<T, Error>;
}

#[derive(Clone)]
pub struct User {
    pub liked_videos: Vec<String>,
    pub following: Vec<String>,
    pub uuid: String,
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
            uuid: uuid.to_string(),
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

impl DatabaseModel<Video> for Video {
    async fn from_db(uuid: &str, db_pool: &MySqlPool) -> Result<Video, Error> {
        let video_row =
            query("SELECT user_id, up_votes, down_votes, views, viewtime_seconds FROM video WHERE uuid = ?")
                .bind(uuid)
                .fetch_one(db_pool)
                .await?;

        let comments = query("SELECT * FROM comment WHERE video_id = ?")
            .bind(uuid)
            .fetch_all(db_pool)
            .await?
            .len();

        let video = Self {
            uuid: uuid.into(),
            user_id: video_row.try_get("user_id")?,
            upvotes: video_row.try_get("up_votes")?,
            downvotes: video_row.try_get("down_votes")?,
            views: video_row.try_get("views")?,
            comments: comments as u32,
            viewtime_seconds: video_row.try_get("viewtime_seconds")?,
        };

        Ok(video)
    }
}
