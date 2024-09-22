use crate::Security::Normal;
use rand::Rng;
use std::time::SystemTime;
use uuid::Uuid;

fn main() {
    const viral_score: f64 = 350_000.;
    let mut test_vid = Video::new(
        Uuid::new_v4(),
        100,
        120,
        Security::Normal,
        SystemTime::now(),
    );
    calc_score(&mut test_vid, viral_score);
    println!("{}", test_vid.score);
    let mut test_vid = Video::new(
        Uuid::new_v4(),
        80,
        1000,
        Security::Normal,
        SystemTime::now(),
    );
    calc_score(&mut test_vid, viral_score);
    println!("{}", test_vid.score);
    let mut test_vid = Video::new(
        Uuid::new_v4(),
        3000,
        16_000,
        Security::Sus2(0.90),
        SystemTime::now(),
    );
    calc_score(&mut test_vid, viral_score);
    println!("{}", test_vid.score);
    let mut test_vid = Video::new(
        Uuid::new_v4(),
        1_000_000,
        35_000_000,
        Security::Normal,
        SystemTime::now(),
    );
    calc_score(&mut test_vid, viral_score);
    println!("{}", test_vid.score);
}

fn _next_vid<'a>(user: User, videos: Vec<Video>) -> &'a str {
    if videos.is_empty() {
        return "No videos present to choose from.";
    }

    return "kp";
}

fn calc_score(video: &mut Video, viral_score: f64) {
    video.score = 1.0;

    video.score += (video.likes as f64 / 10.).powf(1.2);
    video.score += (video.views as f64 / 10.).powf(1.09);

    //Engagement Rate
    video.score *= video.likes as f64 / video.views as f64;

    match video.state {
        State::Boosted => video.score *= 2.,
        State::Private | State::Banned => video.score = 0.,
        State::Normal => {}
    }

    //High Video Risk shoud shadow ban the vid by 50-75%
    match video.security {
        Security::Sus(factor) | Security::Sus2(factor) => video.score *= 1. - factor as f64,
        Normal => {}
    }

    let age = SystemTime::now().duration_since(video.upload_at);

    if let Err(err) = age {
        eprintln!("Error while calculating video age: {err}");
        return;
    }

    let age = (age.unwrap().as_secs() / 60 * 2) as f64;



    // Viral and non-viral balancing
    let virality = video.score / viral_score; // 1.6
    let balancer = video.score * (1. - virality);
    println!("\nBalancer {balancer}, before: {}, virality: {virality}", video.score);

    if virality < 1. {
        video.score += balancer;
    } else {
        video.score += balancer;

    }

    //Add a small amount of randomness
    video.score *= rand::thread_rng().gen_range(0.900..0.950);

    //Make the score smaller for later calculations
    //video.score /= 3.;
}

fn personalize_score(user: User, video: &mut Video) {
    if let Some(following) = user.following {
        if following.contains(&video.user) {
            video.score *= 1.03;
        }
    }
}

struct User<'a> {
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

struct Video {
    uuid: Uuid,
    user: Uuid,
    likes: u32,
    views: u32,
    score: f64,
    security: Security,
    state: State,
    upload_at: SystemTime,
}
enum State {
    Normal,
    Boosted,
    Banned,
    Private,
}

//Change to RiskLevel in code and db
enum Security {
    Normal,
    Sus(f32),
    Sus2(f32),
}

impl Video {
    fn new(user: Uuid, likes: u32, views: u32, security: Security, upload_at: SystemTime) -> Self {
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
