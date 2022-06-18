use serde_json::{Map, Value};

pub fn signal() -> &'static str {
    "signal"
}

pub fn data() -> Value {
    Value::Object(Map::new())
}
