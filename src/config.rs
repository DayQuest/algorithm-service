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
pub const HOST_IP_KEY: &str = "HOST_IP";
pub const HOST_PORT_KEY: &str = "HOST_PORT";

// Tables
pub const DB_VIDEO_TABLE: &str = "video";
pub const DB_LIKED_VIDEOS_TABLE: &str = "liked_videos";
pub const DB_VIEWED_VIDEOS_TABLE: &str = "viewed_video";
// pub const DB_COMMENT_TABLE: &str = "comments";
pub const DB_USER_FOLLOWED_USER_TABLE: &str = "user_followed_users";
pub const VIDEO_READY_STATUS: &str = "3";
pub const DB_USER_LIKED_HASHTAGS_TABLE: &str = "user_liked_hastags";

// Columns
pub const VIEWED_AT_COLUMN: &str = "viewed_at";
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
pub const TIMESTAMP_COLUMN: &str = "timestamp";

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

// Just most important things
pub fn validate(c: &Config) -> Result<(), &str> {
    if c.selecting.max_next_videos_amount
        > c.selecting.next_videos_fetch_amount_matching_hashtag
            + c.selecting.next_videos_fetch_amount_random
    {
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
#[serde(rename_all = "camelCase")]
pub struct ScoringConfig {
    /// The score increases by being multiplied with the upvotes to the power of x.
    /// This value controlls x. Which means: the higher this value is the more it is
    /// going to increase the score, or the upvotes strength is higher.
    pub upvote_exponent: f64,

    /// This is the same as upvote exponent just with the views.
    pub view_exponent: f64,

    /// The score is influenced by a like to view ratio. The more viewer liked the viewer
    /// the higher the score will be increased. This value is being multiplied with the
    /// ratio. This means that this value controls how much the like2view ration influences
    /// the score increasement.
    pub like_2_view_strength: f64,

    /// This is the same as like2view ratio but with the average viewtime per view. The higher
    /// the viewtime, the more people watched the video longer, interpreted as a "better" video.
    /// This value is being multiplied with the ratio so this value controls the strength of the
    /// ratio influence in score increasement.
    pub viewtime_per_view_strength: f64,

    /// The comment count on a video divided by the votes will result in a comments2votes ratio.
    /// This ratio is being multiplied with the score and this value. Means: the higher
    /// this value the more it is going to influence the score increasement.
    pub comments_2_votes_strength: f64,

    /// This controls how strong the upvotes to total votes ratio (so likes and dislikes) influences the score. The higher
    /// this value is the more people that interacted with the video, in form of like or dislike, liked
    /// the video.
    pub upvote_2_totalvotes_strength: f64,

    /// The video scores would be very different. From near 0 to up to millions. To minimize the difference
    /// for a more "fair" this theshold controlls the normalization. If this is on 1.0 = 100% it would result
    /// in pretty much no difference between a pretty viral and 0 likes, 0 views... video. Currently the
    /// score is just used for sorting but later it could also influence the choose-probability which.
    pub normalize_threshold: f64,

    /// PERSONALIZED SCORING:
    /// If the user follows the video creator, the score gets
    /// multiplied by this. If this is higher than 1.0 the p-score (personalized score)
    /// gets higher. If under 1.0 its lower and the probability for that video thrinks.
    pub viewer_following_creator_multiplier: f64,

    /// If the user already liked the video this value gets
    /// multiplied with the pscore. You most likely do not want to show videos
    /// that are already watched again, so this should normally be under 1.0
    pub viewer_liked_video_multiplier: f64,

    /// This value is used as a reference for a score that maps to a "viral"
    /// video. This is just used for the normalization process, it does not directly
    /// influence the scoring.
    pub viral_score: f64,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectingConfig {
    /// The algorithm fetches the hashtags of the videos the user
    /// liked and sorts them so the most frequent hashtags are
    /// at position 1 (i = 0) and the rarest at the last position.
    /// The algorithm than decides for a hashtag which is used to fetch
    /// the hashtag videos (videos that match the selected hashtag, results in
    /// video types the user most likely appreciates!)
    /// This value decides how likely it is to select a hashtag the user likes often
    /// so a hashtag with a high repeat rate.
    pub select_high_freq_hashtag_probability: f64,

    /// This is how many liked videos should be
    /// fetched for hashtag analytics. This should be
    /// way more than actual hashtag video fetch amount.
    pub user_hashtag_fetch_amount: u32,

    /// The algorithm will decide (next_videos_amount times) if the user
    /// gets a score video or a hashtag video. This value will decide
    /// how likely it is that the algorithm choses a HASHTAG video.
    /// The lower this value is the more the algorithm will put in a video
    /// by its score and not by its mathing hashtag with the user liked hashtag
    pub hashtag_2_random_video_probability: f64,

    /// If the algorithm chose a score defined video this will decide
    /// if how likely it is to get a high scored video. The lower this value
    /// the more likely is it get "non-viral" videos. This value
    /// is relevant in combination with the "hashtag_2_random_video_probability"!
    pub high_score_video_probability: f64,

    /// If the algorithm chose a hashtag video it will again choose
    /// if it will use a high scored hashtag video or a low scored hashtag video.
    /// This probabilty decides how likely the user will get a high scored
    /// video (which also matches his hashtag, thats why this value should be low)
    pub high_score_after_hashtag_video_probability: f64,

    /// The max. amount of videos the next videos endpoint will
    /// return. This value needs to be lower than the values underneath.
    pub max_next_videos_amount: u32,

    /// This value decides how many matching hashtag videos will be fetched
    /// for the next videos algorithm
    pub next_videos_fetch_amount_matching_hashtag: u32,

    /// This value decides how many non-matching or random
    /// videos will be fetched from the database.
    /// This should be near the next videos amount.
    pub next_videos_fetch_amount_random: u32,
    
    /// If you have already watched the exact same video this probability decides how
    /// likely it is that the video will even be put into the "algorithm-video-pot".
    /// This helps to prevent video repeatings when having low user counts.
    pub already_watched_video_sort_out_probability: f64,
    
    /// This is the amount of last viewed video uuids that are being
    /// fetched for checking if a video repeats on a users
    /// for you.
    pub already_viewed_videos_fetch_amount: u32,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub config_name: String,
    pub max_dbpool_connections: u32,
    pub scoring: ScoringConfig,
    pub selecting: SelectingConfig,
}