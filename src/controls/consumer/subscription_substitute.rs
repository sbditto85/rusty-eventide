use actix::ActorContext;
use serde_json::{value::Number, Value};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::consumer::{self, subscription::messages};

type Telemetry = Arc<Mutex<HashMap<&'static str, Value>>>;

pub struct SubscriptionSubstitute {
    telemetry: Telemetry,
}

impl SubscriptionSubstitute {
    pub fn new() -> Self {
        Self {
            telemetry: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn telemetry(&self) -> Telemetry {
        self.telemetry.clone()
    }
}

impl actix::Actor for SubscriptionSubstitute {
    type Context = actix::Context<Self>;
}

impl actix::Handler<messages::GetBatch> for SubscriptionSubstitute {
    type Result = ();

    fn handle(&mut self, _msg: messages::GetBatch, ctx: &mut actix::Context<Self>) {
        let mut telemetry = self.telemetry.lock().expect("mutex to not be poisoned");
        telemetry
            .entry(consumer::subscription::telemetry::GET_BATCH)
            .and_modify(|value| match value {
                Value::Number(count) => {
                    *count = Number::from(count.as_i64().map(|c| c + 1).unwrap_or(0))
                }
                _ => *value = Value::Number(Number::from(1)),
            })
            .or_insert(Value::Number(Number::from(1)));

        ()
    }
}

impl actix::Handler<messages::Stop> for SubscriptionSubstitute {
    type Result = ();

    fn handle(&mut self, _msg: messages::Stop, ctx: &mut actix::Context<Self>) {
        ctx.stop()
    }
}
