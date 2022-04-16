use std::collections::HashMap;

use uuid::Uuid;

use crate::session::Session;

pub fn write_random_message_to_random_category() -> String {
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

pub fn write_random_message_to_category(category: &str) {
    let mut session = Session::build().expect("session to build");

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
}

pub fn write_bulk_random_messages_to_category(category: &str, count: u64) {
    for _ in 0..count {
        log::trace!("Writing bulk message...");
        write_random_message_to_category(category);
    }
}

pub fn category_count(category: &str) -> u64 {
    let mut session = Session::build().expect("session to build");

    let rows = session
        .query(
            "select count(1) as number_messages from messages where category(stream_name) = $1::varchar;",
            &[&category],
        )
        .expect("random write to work");

    rows.first()
        .map(|row| {
            let number_messages: i64 = row.get("number_messages");
            number_messages as u64
        })
        .unwrap_or(0)
}

pub fn write_random_message_with_correlation_to_category(category: &str, correlation: &str) {
    let mut session = Session::build().expect("session to build");

    let id = Uuid::new_v4();
    let stream_name = format!("{}-{}", category, id.to_hyphenated().to_string());
    let message_type = "Random";
    let empty_object: HashMap<String, String> = HashMap::new();
    let data = serde_json::to_value(&empty_object).expect("to_string_to_work");
    // correlationStreamName
    let mut correlation_meta_data = HashMap::new();
    correlation_meta_data.insert("correlationStreamName", correlation.to_string());
    let value =
        serde_json::to_value(&correlation_meta_data).expect("metadata to convert to json value");
    let meta_data: Option<serde_json::Value> = Some(value);
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
}

pub fn write_one_random_message_with_data_to_category(data_map: HashMap<&str, &str>) -> String {
    let mut session = Session::build().expect("session to build");

    let category = crate::controls::category::unique_category();

    let id = Uuid::new_v4();
    let stream_name = format!("{}-{}", category, id.to_hyphenated().to_string());
    let message_type = "Random";
    let data = serde_json::to_value(&data_map).expect("to_string_to_work");
    let meta_data: Option<serde_json::Value> = None;
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

pub fn write_one_random_message_for_consumer_and_one_not_to_random_category(
    consumer_group_member: u64,
    consumer_group_size: u64,
) -> String {
    let mut session = Session::build().expect("session to build");

    let category = crate::controls::category::unique_category();

    let id = stream_id_for_consumer_in_group(consumer_group_member, consumer_group_size);
    let stream_name = format!("{}-{}", category, id.to_hyphenated().to_string());
    let message_type = "Random";
    let empty_object: HashMap<String, String> = HashMap::new();
    let data = serde_json::to_value(&empty_object).expect("to_string_to_work");
    let meta_data: Option<serde_json::Value> = None;
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

    let id = stream_id_not_for_consumer_in_group(consumer_group_member, consumer_group_size);
    let stream_name = format!("{}-{}", category, id.to_hyphenated().to_string());
    let message_type = "Random";
    let empty_object: HashMap<String, String> = HashMap::new();
    let data = serde_json::to_value(&empty_object).expect("to_string_to_work");
    let meta_data: Option<serde_json::Value> = None;
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

pub fn stream_id_for_consumer_in_group(
    consumer_group_member: u64,
    consumer_group_size: u64,
) -> Uuid {
    let consumer_group_member = consumer_group_member as i64;
    let consumer_group_size = consumer_group_size as i64;
    let mut session = Session::build().expect("session to build");

    let mut possible_uuid = Uuid::new_v4(); //To keep rust happy I must initialize it before the loop
    let mut belongs = false;
    let mut count = 100;

    while !belongs {
        if count == 0 {
            panic!("Tried too many times to find a valid stream id");
        }

        possible_uuid = Uuid::new_v4();

        let uuid_string = format!("{}", possible_uuid.to_hyphenated_ref());
        let rows = session
            .query(
                "SELECT MOD(@hash_64($3::varchar), $2::bigint) = $1::bigint as belongs;",
                &[&consumer_group_member, &consumer_group_size, &uuid_string],
            )
            .expect("the mod query to run");

        belongs = rows[0].get::<&str, bool>("belongs");
        log::debug!("UUID: {} => {}", uuid_string, belongs);

        count -= 1;
    }

    possible_uuid
}

pub fn stream_id_not_for_consumer_in_group(
    consumer_group_member: u64,
    consumer_group_size: u64,
) -> Uuid {
    let consumer_group_member = consumer_group_member as i64;
    let consumer_group_size = consumer_group_size as i64;
    let mut session = Session::build().expect("session to build");

    let mut possible_uuid = Uuid::new_v4(); //To keep rust happy I must initialize it before the loop
    let mut belongs = true;
    let mut count = 100;

    while belongs {
        if count == 0 {
            panic!("Tried too many times to find a valid stream id");
        }

        possible_uuid = Uuid::new_v4();

        let uuid_string = format!("{}", possible_uuid.to_hyphenated_ref());
        let rows = session
            .query(
                "SELECT MOD(@hash_64($3::varchar), $2::bigint) = $1::bigint as belongs;",
                &[&consumer_group_member, &consumer_group_size, &uuid_string],
            )
            .expect("the mod query to run");

        belongs = rows[0].get::<&str, bool>("belongs");
        log::debug!("UUID: {} => {}", uuid_string, belongs);

        count -= 1;
    }

    possible_uuid
}

pub fn enable_condition_for_session(session: &mut Session) {
    session
        .query(
            "SELECT set_config('message_store.sql_condition', 'on', false);",
            &[],
        )
        .expect("the mod query to run");
}
