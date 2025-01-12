
use sqlx::{query_scalar, Error, MySqlPool};

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
        todo!()
    }
}
