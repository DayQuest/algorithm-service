use algorythm::{Security, Video};
use std::time::SystemTime;
use uuid::Uuid;

mod algorythm;
fn main() {
    const VIRAL_SCORE: f64 = 10_000.;
    algorythm::calc_score(&mut Video::new(
        Uuid::new_v4(),
        0, //likes
        12, //views
        Security::Normal,
        SystemTime::now(),
    ), VIRAL_SCORE);

    algorythm::calc_score(&mut Video::new(
        Uuid::new_v4(),
        1500, //likes
        120_000, //views
        Security::Normal,
        SystemTime::now(),
    ), VIRAL_SCORE);

    algorythm::calc_score(&mut Video::new(
        Uuid::new_v4(),
        200_000, //likes
        12_000_000, //views
        Security::Normal,
        SystemTime::now(),
    ), VIRAL_SCORE);
}