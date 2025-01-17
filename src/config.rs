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
pub const JWT_SECRET_KEY: &str = "JWT_SECRET";
pub const INTERNAL_SECRET_KEY: &str = "INTERNAL_SECRET";
pub const DATABASE_CONN_URL_KEY: &str = "DATABASE_CONNECTION_URL";

//Tables
pub const DB_VIDEO_TABLE: &str = "video";
pub const DB_COMMENT_TABLE: &str = "comment";
pub const DB_LIKED_VIDEOS_TABLE: &str = "liked_videos";
pub const DB_USER_FOLLOWED_USER_TABLE: &str = "user_followed_user";

//Columns
pub const USER_ID_COLUMN: &str = "user_id";
pub const FOLLOWED_USERS_COLUMN: &str = "followed_users";
pub const UUID_COLUMN: &str = "uuid";

//Video Status
pub const VIDEO_STATUS_COLUMN: &str = "status";
pub const VIDEO_READY_STATUS: &str = "3";

pub const VIDEO_ID_COLUMN: &str = "video_id";
pub const VIDEO_UP_VOTES_COLUMN: &str = "up_votes";
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
pub fn validate(c: &Config) -> Result<(), &str>{
    if c.next_videos_amount > c.next_videos_fetch_amount {
        return Err("Next Video amount must be higher than the Next Video Fetch Amount!");
    }

    if c.viral_score <= 0. {
        return Err("Viral Score must be higher than 0.00");
    }

    if c.high_score_video_probability <= 0. {
        return Err("High Score Prob. must be higher than 0.00");
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub config_name: String,
    pub viral_score: f64,
    pub high_score_video_probability: f64,
    pub upvote_exponent: f64,
    pub view_exponent: f64,
    pub upvote_2_totalvotes_ratio_exponent: f64,
    pub normalize_threshold: f64,
    pub viewer_following_creator_multiplier: f64,
    pub viewer_liked_video_multiplier: f64,
    pub next_videos_amount: u32,
    pub next_videos_fetch_amount: u32,
}

