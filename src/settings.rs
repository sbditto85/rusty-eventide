const POSITION_UPDATE_INTERVAL_DEFAULT: u64 = 100;
const MESSAGE_DB_URL_DEFAULT: &'static str = "postgres://message_store@localhost/message_store";

#[derive(Debug)]
pub struct Settings {
    pub position_update_interval: u64,
    pub message_db_url: String,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            position_update_interval: POSITION_UPDATE_INTERVAL_DEFAULT,
            message_db_url: MESSAGE_DB_URL_DEFAULT.to_string(),
        }
    }

    // TODO: impl this
    pub fn build() -> Self {
        // todo!("grab from env and then override default")

        Self {
            position_update_interval: POSITION_UPDATE_INTERVAL_DEFAULT,
            message_db_url: MESSAGE_DB_URL_DEFAULT.to_string(),
        }
    }
}
