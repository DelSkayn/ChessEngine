pub fn outcome(rating: f64, opponent: f64) -> f64 {
    dbg!(rating, opponent);
    1.0 / (1.0 + 10.0f64.powf((opponent - rating) / 400.0))
}

pub fn new_rating(rating: f64, expected: f64, outcome: f64, k: f64) -> f64 {
    rating + k * (outcome - expected)
}

pub fn update(rating: &mut f64, score: f64, opponent: f64, k: f64) {
    let expected = outcome(*rating, opponent);
    *rating = new_rating(*rating, expected, score, k);
}
