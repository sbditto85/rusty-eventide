const POSITION_UPDATE_INTERVAL_DEFAULT: u64 = 100;
const MESSAGE_DB_URL_DEFAULT: &'static str = "postgres://message_store@localhost/message_store";

const BATCH_SIZE_DEFAULT: Option<u64> = None; //1000 for messagedb

#[derive(Debug)]
pub struct Settings {
    pub position_update_interval: u64,
    pub message_db_url: String,
    pub batch_size: Option<u64>,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            position_update_interval: POSITION_UPDATE_INTERVAL_DEFAULT,
            message_db_url: MESSAGE_DB_URL_DEFAULT.to_string(),
            batch_size: BATCH_SIZE_DEFAULT,
        }
    }

    // TODO: impl this
    pub fn build() -> Self {
        // todo!("grab from env and then override default")

        Self {
            position_update_interval: POSITION_UPDATE_INTERVAL_DEFAULT,
            message_db_url: MESSAGE_DB_URL_DEFAULT.to_string(),
            batch_size: BATCH_SIZE_DEFAULT,
        }
    }
}
