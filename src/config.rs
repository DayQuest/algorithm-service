use figment::{providers::{Format, Json}, Figment};
use log::info;
use serde::Deserialize;



pub fn load() -> Config {
    let config = Figment::new()
    .merge(Json::file("config.json"));

    let config: Config = config.extract().expect("Failed to load config..");

    info!("Loaded config: {}", config.config_name);
    config
}

#[derive(Deserialize)]
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
