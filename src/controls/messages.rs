use std::collections::HashMap;

use uuid::Uuid;

use crate::messaging::MessageData;
use crate::session::Session;

pub fn example() -> Vec<MessageData> {
    vec![
        MessageData { global_position: 0 },
        MessageData { global_position: 1 },
    ]
}

pub fn write_random_to_random_category() -> String {
    let mut session = Session::build().expect("session to build");

    let category = crate::controls::category::unique_category();

    let id = Uuid::new_v4();
    let stream_name = format!("{}-{}", category, id.to_hyphenated().to_string());
    let message_type = "Random";
    let empty_object: HashMap<String, String> = HashMap::new();
    let data = serde_json::to_value(&empty_object).expect("to_string_to_work");
    let meta_data: Option<serde_json::Value> = None; // Some(serde_json::to_value(&empty_object).expect("to_string_to_work"));
    let expected_version = -1i64;

    session
        .query(
            "SELECT write_message($1::varchar, $2::varchar, $3::varchar, $4::jsonb, $5::jsonb, $6::bigint);",
            &[
                &id.to_hyphenated().to_string(),
                &stream_name,
                &message_type,
                &data,
                &meta_data,
                &expected_version,
            ],
        )
        .expect("random write to work");

    category
}
