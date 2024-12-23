use rand::Rng;
use sqlx::{query_scalar, Error, MySqlPool};

const VIRAL_SCORE: f64 = 10_000.;

pub fn _next_vid<'a>(_user: User, videos: Vec<Video>) -> &'a str {
    if videos.is_empty() {
        return "No videos present to choose from.";
    }

    return "Not implemented";
}

pub fn calc_score(video: &Video) -> f64 {
    let mut score = 1.;

    score += (video.upvotes as f64 / 10.).powf(1.11);
    score += (video.views as f64 / 10.).powf(1.05);

    if video.views != 0 {
        score *= video.upvotes as f64 / video.views as f64;
    }

    if video.views != 0 {
        let total_votes = video.upvotes + video.downvotes;
        if total_votes != 0 {
            let upvote_ratio = video.upvotes as f64 / total_votes as f64;
            score *= upvote_ratio.powf(1.2);
            
            let downvote_impact = video.downvotes as f64 / video.views as f64;
            score *= (1.0 - downvote_impact).max(0.5);
        }
    }

    // TODO: Add video age

    normalize_score(&mut score, &VIRAL_SCORE, 0.995);

    /*match &video.risk_level {
        RiskLevel::Sus => score *= 0.5,
        RiskLevel::Sus2 => score *= 0.2,
        RiskLevel::Normal => {}
    }*/

    score *= rand::thread_rng().gen_range(0.240..0.250);
    score /= 3.;

    score
}

fn normalize_score(score: &mut f64, target: &f64, threshold: f64) {
    let threshold = threshold.max(0.0).min(1.0);
    let ratio = *score / target;

    if ratio > 1.0 {
        *score = target * (1.0 + (ratio - 1.0) * (1.0 - threshold));
    } else {
        *score = target * (1.0 - (1.0 - ratio) * (1.0 - threshold));
    }
}

pub fn personalize_score(user: User, video: &Video) -> f64 {
    let mut score = video.score;
    if user.following.contains(&video.user_id) {
        score *= 1.03;
    }

    //Do not wanna show exact same videos
    if user.liked_videos.contains(&video.uuid) {
        score *= 0.96;
    }

    score
}

pub struct User {
    pub username: String,
    pub liked_videos: Vec<String>,
    pub following: Vec<String>,
    pub uuid: String,
}

impl User {
    pub async fn from_db(uuid: String, db_pool: MySqlPool) -> Result<User, Error> {
        let username = query_scalar("SELECT username FROM users WHERE uuid = ?")
            .bind(&uuid)
            .fetch_one(&db_pool)
            .await?;

        let liked_videos: Vec<String> =
            query_scalar("SELECT * FROM liked_videos WHERE user_id = ?")
                .bind(&uuid)
                .fetch_all(&db_pool)
                .await?;

        let following: Vec<String> =
            query_scalar("SELECT * FROM user_followed_users WHERE user_uuid = ?")
                .bind(&uuid)
                .fetch_all(&db_pool)
                .await?;

        Ok(Self {
            username,
            liked_videos,
            following,
            uuid,
        })
    }
}

#[derive(sqlx::FromRow)]
pub struct Video {
    pub uuid: String,
    pub user_id: String,
    pub upvotes: u32,
    pub downvotes: u32,
    pub views: u32,
    pub score: f64,
}
