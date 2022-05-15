use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::controls;
use crate::messaging::category::{Category, CategoryTypes};

pub fn example() -> Category {
    example_category(None, &[], true)
}

fn example_category(category: Option<&str>, types: CategoryTypes, randomize: bool) -> Category {
    let mut category = category.unwrap_or("test").to_string();
    if randomize {
        let random_part: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();
        category = format!("{}{}XX", category, random_part);
    }

    let stream_id = None;
    controls::messages::stream_name::stream_name(category, stream_id, types)
}

pub fn unique_category() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}
