use crate::messaging::MessageData;

pub mod category;
pub mod postgres;
pub mod stream_name;

pub fn beginning_global_position() -> u64 {
    1
}

pub fn example() -> Vec<MessageData> {
    let starting_position = beginning_global_position();
    vec![
        MessageData {
            global_position: starting_position,
        },
        MessageData {
            global_position: starting_position + 1,
        },
    ]
}
