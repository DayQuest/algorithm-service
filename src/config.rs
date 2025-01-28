use std::{
    fs::{self, File},
    io::{Error, Write},
};

use figment::{
    providers::{Format, Json},
    Figment,
};
use log::info;
use serde::{Deserialize, Serialize};

const FILE_PATH: &str = "config.json";
pub const JWT_SECRET_KEY: &str = "JWT_SECRETJWT_SECRET";
pub const INTERNAL_SECRET_KEY: &str = "INTERNAL_SECRET";
pub const DATABASE_CONN_URL_KEY: &str = "DATABASE_CONNECTION_URL";
pub const HOST_IP_KEY: &str = "HOST_IP";
pub const HOST_PORT_KEY: &str = "HOST_PORT";

//Tables
pub const DB_VIDEO_TABLE: &str = "video";
pub const DB_LIKED_VIDEOS_TABLE: &str = "liked_videos";
//pub const DB_COMMENT_TABLE: &str = "comments";
pub const DB_USER_FOLLOWED_USER_TABLE: &str = "user_followed_users";
pub const VIDEO_READY_STATUS: &str = "3";

//Columns
pub const USER_ID_COLUMN: &str = "user_id";
pub const VIDEO_STATUS_COLUMN: &str = "status";
pub const FOLLOWED_USERS_COLUMN: &str = "followed_users";
pub const UUID_COLUMN: &str = "uuid";
pub const VIDEO_COMMENTS_COLUMN: &str = "comments";
pub const VIDEO_ID_COLUMN: &str = "video_id";
pub const VIDEO_UP_VOTES_COLUMN: &str = "up_votes";
pub const VIDEO_HASHTAGS_COLUMN: &str = "hashtags";
pub const VIDEO_DOWN_VOTES_COLUMN: &str = "down_votes";
pub const VIDEO_VIEWS_COLUMN: &str = "views";
pub const VIDEO_VIEWTIME_COLUMN: &str = "viewtime_seconds";

pub fn load() -> Config {
    let config = Figment::new().merge(Json::file(FILE_PATH));

    let config: Config = config.extract().expect("Failed to load config..");

    info!("Loaded config: {}", config.config_name);
    config
}

pub fn overwrite(content: String) -> Result<(), Error> {
    fs::remove_file(FILE_PATH)?;

    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(FILE_PATH)
        .unwrap();

    file.write_all(content.as_bytes())?;
    Ok(())
}

//Just most important things
pub fn validate(c: &Config) -> Result<(), &str> {
    if c.selecting.next_videos_amount > c.selecting.next_videos_fetch_amount_matching_hashtag + c.selecting.next_videos_fetch_amount_random {
        return Err("Next Video amount must be higher than the Next Video Fetch Amount!");
    }

    if c.scoring.viral_score <= 0. {
        return Err("Viral Score must be higher than 0.00");
    }

    if c.selecting.high_score_video_probability <= 0. {
        return Err("High Score Prob. must be higher than 0.00");
    }

    if c.scoring.like_2_view_strength <= 0. {
        return Err("Like 2 view strength must be higher than 0.00");
    }

    if c.scoring.comments_2_votes_strength <= 0. {
        return Err("Comments 2 votes strength must be higher than 0.00");
    }

    if c.scoring.viewtime_per_view_strength <= 0. {
        return Err("Viewtime per view strength must be higher than 0.00");
    }

    if c.scoring.upvote_2_totalvotes_strength <= 0. {
        return Err("Upvotes 2 totalvotes strength must be higher than 0.00");
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ScoringConfig{
    pub upvote_exponent: f64,
    pub view_exponent: f64,
    pub like_2_view_strength: f64,
    pub viewtime_per_view_strength: f64,
    pub comments_2_votes_strength: f64,
    pub upvote_2_totalvotes_strength: f64,
    pub normalize_threshold: f64,
    pub viewer_following_creator_ratio_exponent: f64,
    pub viewer_liked_video_ratio_exponent: f64,
    pub viral_score: f64,
}


#[derive(Deserialize, Serialize, Clone)]
pub struct SelectingConfig {
    pub hashtag_video_probability: f64,
    pub high_score_video_probability: f64,
    pub next_videos_amount: u32,
    pub next_videos_fetch_amount_matching_hashtag: u32,
    pub next_videos_fetch_amount_random: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub config_name: String,
    pub max_dbpool_connections: u32,
    pub scoring: ScoringConfig,
    pub selecting: SelectingConfig,
}
