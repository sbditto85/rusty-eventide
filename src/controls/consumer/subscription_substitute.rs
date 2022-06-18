use actix::ActorContext;
use serde_json::{value::Number, Value};

use crate::consumer::{self, subscription::messages};
use crate::telemetry::Telemetry;

pub struct SubscriptionSubstitute {
    telemetry: Telemetry,
}

impl SubscriptionSubstitute {
    pub fn new() -> Self {
        Self {
            telemetry: Telemetry::new(),
        }
    }

    pub fn telemetry(&mut self) -> &mut Telemetry {
        &mut self.telemetry
    }
}

impl actix::Actor for SubscriptionSubstitute {
    type Context = actix::Context<Self>;
}

impl actix::Handler<messages::GetBatch> for SubscriptionSubstitute {
    type Result = ();

    fn handle(&mut self, _msg: messages::GetBatch, _ctx: &mut actix::Context<Self>) {
        let signal = consumer::subscription::telemetry::GET_BATCH;
        self.telemetry
            .record_data(signal, Value::Number(Number::from(1)));
    }
}

impl actix::Handler<messages::Stop> for SubscriptionSubstitute {
    type Result = ();

    fn handle(&mut self, _msg: messages::Stop, ctx: &mut actix::Context<Self>) {
        ctx.stop()
    }
}

pub fn batch() -> Vec<()> {
    vec![(), (), ()]
}
