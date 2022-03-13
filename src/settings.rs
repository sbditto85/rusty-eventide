const POSITION_UPDATE_INTERVAL_DEFAULT: u64 = 100;

#[derive(Debug)]
pub struct Settings {
    pub position_update_interval: u64,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            position_update_interval: POSITION_UPDATE_INTERVAL_DEFAULT,
        }
    }

    // TODO: impl this
    pub fn build() -> Self {
        // todo!("grab from env and then override default")
        Self {
            position_update_interval: POSITION_UPDATE_INTERVAL_DEFAULT,
        }
    }
}
