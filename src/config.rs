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
use tokio::fs::read;

const FILE_PATH: &str = "config.json";

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

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub config_name: String,
    pub viral_score: f64,
    pub upvote_exponent: f64,
    pub view_exponent: f64,
    pub upvote_2_totalvotes_ratio_exponent: f64,
    pub normalize_threshold: f64,
    pub viewer_following_creator_multiply: f64,
    pub viewer_liked_video_multiply: f64,
    pub next_videos_amount: f64,
}
