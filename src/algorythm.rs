use std::{cmp::Ordering, sync::Arc, time::SystemTime};

use rand::Rng;
use uuid::Uuid;

pub fn _next_vid<'a>(user: User, videos: Vec<Video>) -> &'a str {
    if videos.is_empty() {
        return "No videos present to choose from.";
    }


    return "Not implemented";
}

pub fn calc_score(video: &mut Video, viral_score: f64) {
    video.score = 1.0;

    video.score += (video.likes as f64 / 10.).powf(1.2);
    video.score += (video.views as f64 / 10.).powf(1.09);

    //Engagement Rate
    if video.views != 0 {
        video.score *= video.likes as f64 / video.views as f64;
    }

    let age = SystemTime::now().duration_since(video.upload_at);

    if let Err(err) = age {
        eprintln!("Error while calculating video age: {err}");
        return;
    }

    //Impement video age
    let _age = (age.unwrap().as_secs() / 60 * 2) as f64;

    normalize_score(&mut video.score, &viral_score, 0.005);

    match video.state {
        State::Boosted => video.score *= 2.,
        State::Private | State::Banned => video.score = 0.,
        State::Normal => {}
    }

    match &video.security {
        Security::Sus => video.score *= 0.5,
        Security::Sus2 => video.score *= 0.2,
        Security::Normal => {}
    }

    video.score *= rand::thread_rng().gen_range(0.940..0.950);
    video.score /= 3.;

    println!("Score: {}", video.score);
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
pub fn personalize_score(user: User, video: &mut Video) {
    if let Some(following) = user.following {
        if following.contains(&video.user) {
            video.score *= 1.03;
        }
    }

    //Do not wanna show exaxt same videos
    if let Some(liked_vids) = user.liked_videos {
        if liked_vids.contains(&video.uuid) {
            video.score *= 0.96;
        }
    }
}

pub struct User<'a> {
    username: &'a str,
    liked_videos: Option<Vec<Uuid>>,
    following: Option<Vec<Uuid>>,
    watched: Option<Vec<Uuid>>,
    uuid: Uuid,
}

impl<'a> User<'a> {
    fn new(
        username: &'a str,
        liked_videos: Option<Vec<Uuid>>,
        following: Option<Vec<Uuid>>,
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

pub struct Video {
    uuid: Uuid,
    user: Uuid,
    likes: u32,
    views: u32,
    score: f64,
    security: Security,
    state: State,
    upload_at: SystemTime,
}
pub enum State {
    Normal,
    Boosted,
    Banned,
    Private,
}

//Change to RiskLevel in code and db
pub enum Security {
    Normal,
    Sus,
    Sus2,
}

impl Video {
    pub fn new(user: Uuid, likes: u32, views: u32, security: Security, upload_at: SystemTime) -> Self {
        Video {
            uuid: Uuid::new_v4(),
            user,
            likes,
            security,
            score: 1.0,
            views,
            state: State::Normal,
            upload_at,
        }
    }
}
