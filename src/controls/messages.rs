use crate::messaging::MessageData;
use crate::session::Session;

pub fn example() -> Vec<MessageData> {
    vec![
        MessageData { global_position: 0 },
        MessageData { global_position: 1 },
    ]
}

pub fn write_random(category: &str) {
    let session = Session::build();
}
