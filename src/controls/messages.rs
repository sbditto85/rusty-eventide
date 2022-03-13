use crate::messaging::MessageData;

pub fn example() -> Vec<MessageData> {
    vec![
        MessageData { global_position: 0 },
        MessageData { global_position: 1 },
    ]
}
