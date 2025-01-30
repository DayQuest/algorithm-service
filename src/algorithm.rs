use std::{collections::HashMap, error::Error, time::Instant};

use crate::{
    config::Config,
    database::{self, User, Video},
};
use log::debug;
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng};
use sqlx::MySqlPool;

fn random_bool(probability_true: f64) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen::<f64>() < (probability_true)
}

fn weighted_random<T>(vec: &Vec<T>, decay_factor: f64) -> Option<T>
where
    T: Clone,
{
    let len = vec.len();
    if len == 0 {
        return None;
    }

    let weights: Vec<f64> = (0..len)
        .map(|i| {
            let weight = decay_factor.powi(i as i32);
            weight
        })
        .collect();

    let dist = WeightedIndex::new(&weights).unwrap();
    let mut rng = thread_rng();
    let index = dist.sample(&mut rng);

    Some(vec[index].clone())
}

fn score_sort_random_videos(videos: Vec<Video>, user: &User, config: &Config) -> Vec<Video> {
    let mut scored_random_vids = videos
        .into_iter()
        .map(|mut video| {
            video.score = score_video_personalized(user, &video, config);
            video
        })
        .collect::<Vec<Video>>();

    //Sort => lowest first.
    scored_random_vids.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    scored_random_vids
}

fn sort_user_hashtags_by_frequency(user: &User) -> Vec<String> {
    let mut frequency_map: HashMap<&String, usize> = HashMap::new();
    
    for hashtag in &user.last_hashtags {
        *frequency_map.entry(hashtag).or_insert(0) += 1;
    }
    
    let mut sorted_hashtags: Vec<_> = frequency_map.into_iter().collect();
    sorted_hashtags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    
    sorted_hashtags.into_iter().map(|(hashtag, _)| hashtag.clone()).collect()
}


pub async fn next_videos(
    user: &User,
    config: &Config,
    db_pool: &MySqlPool,
) -> Result<Vec<Video>, Box<dyn Error>> {
    let start_time = Instant::now();
    debug!("User: likes: {:?}, followed: {:?}, hashtags: {:?}", user.liked_videos, user.following, user.last_hashtags);
    let selected_hashtag = weighted_random(
        &sort_user_hashtags_by_frequency(user), // Sorts by frequency, so i = 0 is the most "liked" hashtag
        config.selecting.user_hashtag_decay_factor,
    );

    let fetched_videos = database::fetch_next_videos(
        config,
        selected_hashtag.clone().unwrap_or_else(|| "/".into()),
        db_pool,
    )
    .await?;

    let hashtag_videos = fetched_videos.1;
    let sorted_rand_scored_vids = score_sort_random_videos(fetched_videos.0, user, config);

    let mut final_sort: Vec<Video> = Vec::new();

    let mut high_score_video_chosen = 0;
    for i in 0..sorted_rand_scored_vids.len() + hashtag_videos.len() {
        if i >= config.selecting.next_videos_amount.try_into().unwrap() {
            break;
        }

        if selected_hashtag.is_some() && random_bool(config.selecting.hashtag_2_random_video_probability) {
            // Hashtag Video
            let chosen = weighted_random(
                &hashtag_videos,
                config.selecting.select_high_freq_hashtag_probability,
            )
            .unwrap();

            final_sort.push(chosen);
        } else {
            // Non-Hashtag or random video
            if random_bool(config.selecting.high_score_video_probability) {
                //Put a high scored video in
                let video = sorted_rand_scored_vids
                    .get(sorted_rand_scored_vids.len() - high_score_video_chosen - 1)
                    .unwrap();
                final_sort.push(video.clone());
                high_score_video_chosen += 1;
            } else {
                final_sort.push(sorted_rand_scored_vids.get(i).unwrap().clone());
            }
        }
    }

    debug!(
        "Next Video Selecting took: {} ms",
        start_time.elapsed().as_millis()
    );
    Ok(final_sort)
}

pub fn score_video(video: &Video, config: &Config) -> f64 {
    let mut score = 1.;

    //Score up with likes
    score += (video.upvotes as f64 / 10.).powf(config.scoring.upvote_exponent);

    //Score up with views
    score += (video.views as f64 / 10.).powf(config.scoring.view_exponent);

    //Score multiplied by like-2-view ratio
    if video.views != 0 {
        score *= (video.upvotes as f64 / video.views as f64) * config.scoring.like_2_view_strength;
    }

    //Multiply score by upvote-2-totalvotes ratio
    //Multiply score by downvote-2-views impact
    //Multiply score by avg. viewtime per view
    let total_votes = video.upvotes + video.downvotes;
    if video.views != 0 {
        score *= (video.viewtime_seconds as f64 / video.views as f64)
            * config.scoring.viewtime_per_view_strength;

        if total_votes != 0 {
            let upvote_ratio = video.upvotes as f64 / total_votes as f64;
            score *= upvote_ratio * config.scoring.upvote_2_totalvotes_strength;

            let downvote_impact = video.downvotes as f64 / video.views as f64;
            score *= (1.0 - downvote_impact).max(0.5);
        }
    }

    if video.comments != 0 {
        //lower = better
        let comments_2_votes_ratio = total_votes as f64 / video.comments as f64;
        score *= (1. / comments_2_votes_ratio) * config.scoring.comments_2_votes_strength;
    }

    normalize_score(
        &mut score,
        &config.scoring.viral_score,
        config.scoring.normalize_threshold,
    );

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
        score *= config.scoring.viewer_following_creator_multiplier;
    }

    //Do not wanna show exact same videos
    if user.liked_videos.contains(&video.uuid) {
        score *= config.scoring.viewer_liked_video_multiplier;
    }

    score
}
