use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn example() -> String {
    //TODO:
    unique_category()
}

pub fn unique_category() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}
