use crate::{
    config::Config,
    database::{self, User, Video},
};
use rand::Rng;
use sqlx::{Error, MySqlPool};

fn random_bool(probability_true: f64) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < (probability_true)
}

pub async fn next_videos(
    user: &User,
    config: &Config,
    db_pool: &MySqlPool,
) -> Result<Vec<Video>, Error> {
    //TODO: Get videos from database (later with a hashtag algorithm which filters about x%)
    //TODO: Personalized score the videos
    //TODO: Order with a pattern: Video with low score, sometimes high score, after high score x% low score
    let fetched_videos =
        database::get_random_videos(config.next_videos_amount, false, db_pool).await?;

    //Score Videos
    let mut scored_personalized_videos = fetched_videos
        .into_iter()
        .map(|mut video| {
            video.score = score_video_personalized(user, &video, config);
            video // Return the owned `Video`
        })
        .collect::<Vec<Video>>();

    //Sort => lowest first.
    scored_personalized_videos.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let mut final_sort: Vec<Video> = Vec::new();

    for (i, _) in scored_personalized_videos.iter().enumerate() {
        if random_bool(config.high_score_video_probability) {
            //Put a high scored video in
            let video = scored_personalized_videos
                .get(scored_personalized_videos.len() - i)
                .unwrap();
            final_sort.push(video.clone());
        } else {
            final_sort.push(scored_personalized_videos.get(i).unwrap().clone());
        }
    }


    Ok(final_sort)
}

pub fn score_video(video: &Video, config: &Config) -> f64 {
    let mut score = 1.;

    //Score up with likes
    score += (video.upvotes as f64 / 10.).powf(config.upvote_exponent);

    //Score up with views
    score += (video.views as f64 / 10.).powf(config.view_exponent);

    //Score multiplied add by like-2-view ratio
    if video.views != 0 {
        score *= video.upvotes as f64 / video.views as f64;
    }

    //Multiply score by upvote-2-totalvotes ratio
    //Multiply score by downvote-2-views impact
    //Multiply score by avg. viewtime per view
    let total_votes = video.upvotes + video.downvotes;
    if video.views != 0 {
        score *= video.viewtime_seconds as f64 / video.views as f64;

        if total_votes != 0 {
            let upvote_ratio = video.upvotes as f64 / total_votes as f64;
            score *= upvote_ratio.powf(config.upvote_2_totalvotes_ratio_exponent);

            let downvote_impact = video.downvotes as f64 / video.views as f64;
            score *= (1.0 - downvote_impact).max(0.5);
        }
    }

    if video.comments != 0 {
        let comments_2_votes_ratio = total_votes as f64 / video.comments as f64;
        score *= 1. / comments_2_votes_ratio;
    }

    normalize_score(&mut score, &config.viral_score, config.normalize_threshold);
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

pub fn score_video_personalized(user: &User, video: &Video, config: &Config) -> f64 {
    let mut score = score_video(video, config);
    if user.following.contains(&video.user_id) {
        score *= config.viewer_following_creator_multiplier;
    }

    //Do not wanna show exact same videos
    if user.liked_videos.contains(&video.uuid) {
        score *= config.viewer_liked_video_multiplier;
    }

    score
}
