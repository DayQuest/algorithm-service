use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


const VIRAL_SCORE: f64 = 10_000.;

pub fn _next_vid<'a>(user: User, videos: Vec<Video>) -> &'a str {
    if videos.is_empty() {
        return "No videos present to choose from.";
    }

    return "Not implemented";
}

pub fn calc_score(video: &Video) -> f64 {
    let mut score = 1.;

    score += (video.likes as f64 / 10.).powf(1.11);
    score += (video.views as f64 / 10.).powf(1.05);

    //Engagement Rate
    if video.views != 0 {
        score *= video.likes as f64 / video.views as f64;
    }

    //TODO: Add video age

    normalize_score(&mut score, &VIRAL_SCORE, 0.995);

    match video.state {
        State::Boosted => score *= 2.,
        State::Private | State::Banned => score = 0.,
        State::Normal => {}
    }

    match &video.security {
        Security::Sus => score *= 0.5,
        Security::Sus2 => score *= 0.2,
        Security::Normal => {}
    }

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
    if let Some(following) = user.following {
        if following.contains(&video.user) {
            score *= 1.03;
        }
    }

    //Do not wanna show exact same videos
    if let Some(liked_vids) = user.liked_videos {
        if liked_vids.contains(&video.uuid) {
            score *= 0.96;
        }
    }

    score
}

pub struct User<'a> {
    username: &'a str,
    liked_videos: Option<Vec<String>>,
    following: Option<Vec<String>>,
    watched: Option<Vec<String>>,
    uuid: Uuid,
}

impl<'a> User<'a> {
    fn new(
        username: &'a str,
        liked_videos: Option<Vec<String>>,
        following: Option<Vec<String>>,
    ) -> Self {
        User {
            username,
            liked_videos,
            following,
            watched: None,
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Video {
    pub uuid: String,
    pub user: String,
    pub likes: u32,
    pub views: u32,
    pub score: f64,
    pub security: Security,
    pub state: State,
}

#[derive(Deserialize, Serialize)]
pub enum State {
    Normal,
    Boosted,
    Banned,
    Private,
}

//Change to RiskLevel in code and db
#[derive(Deserialize, Serialize)]
pub enum Security {
    Normal,
    Sus,
    Sus2,
}

impl Video {
    pub fn new(
        user: Uuid,
        likes: u32,
        views: u32,
        security: Security,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            user: user.to_string(),
            likes,
            security,
            score: 1.0,
            views,
            state: State::Normal,
        }
    }
}
