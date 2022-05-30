use actix::{dev::ToEnvelope, Actor as ActixActor, ActorContext, Addr};

use crate::consumer::subscription::messages as subscription_messages;

pub mod messages;

pub struct Actor<S: actix::Actor> {
    subscription_addr: Addr<S>,
}

impl<S: actix::Actor> Actor<S> {
    pub fn new(subscription_addr: Addr<S>) -> Self {
        Self {
            subscription_addr: subscription_addr,
        }
    }
}

impl<S> ActixActor for Actor<S>
where
    S: actix::Actor,
    <S as actix::Actor>::Context: ToEnvelope<S, subscription_messages::GetBatch>,
    S: actix::Handler<subscription_messages::GetBatch>,
{
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscription_addr
            .do_send(subscription_messages::GetBatch);
    }
}

impl<S> actix::Handler<messages::Stop> for Actor<S>
where
    S: actix::Actor,
    <S as actix::Actor>::Context: ToEnvelope<S, subscription_messages::GetBatch>,
    S: actix::Handler<subscription_messages::GetBatch>,
{
    type Result = ();

    fn handle(&mut self, _msg: messages::Stop, ctx: &mut actix::Context<Self>) {
        ctx.stop()
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    use serde_json::Value;

    use crate::consumer;
    use crate::controls;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[actix::test]
    async fn should_call_subscription_get_batch_on_start() {
        init();

        let subscription =
            controls::consumer::subscription_substitute::SubscriptionSubstitute::new();
        let telemetry = subscription.telemetry();
        let subscription_addr = subscription.start();

        let addr = Actor::new(subscription_addr.clone()).start();
        addr.send(messages::Stop).await.expect("send to work");
        subscription_addr
            .send(subscription_messages::Stop)
            .await
            .expect("send to work");

        let batch_called = 1;
        let tel = telemetry.lock().expect("mutex to not be poisoned");
        if let Some(Value::Number(count)) = tel.get(consumer::subscription::telemetry::GET_BATCH) {
            assert_eq!(count.as_i64().unwrap_or(0), batch_called);
        } else {
            panic!("No record of get batch recorded");
        }
    }
}
