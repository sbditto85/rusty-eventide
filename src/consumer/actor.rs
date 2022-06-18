use actix::{
    dev::ToEnvelope,
    Actor as ActixActor,
    ActorContext,
    // ActorFutureExt,
    Addr,
    // AsyncContext,
    // ResponseActFuture,
    // WrapFuture,
};

use crate::{consumer::subscription::messages as subscription_messages, telemetry::Telemetry};

pub mod messages;
pub mod telemetry;

pub struct Actor<S: actix::Actor> {
    subscription_addr: Addr<S>,
    telemetry: Telemetry,
    pre_fetch_queue: Vec<()>,
}

impl<S: actix::Actor> Actor<S> {
    pub fn new(subscription_addr: Addr<S>) -> Self {
        Self {
            subscription_addr,
            telemetry: Telemetry::new(),
            pre_fetch_queue: Vec::new(),
        }
    }

    pub fn telemetry(&mut self) -> &mut Telemetry {
        &mut self.telemetry
    }

    pub fn pre_fetch_queue(&self) -> &Vec<()> {
        &self.pre_fetch_queue
    }
}

impl<S> ActixActor for Actor<S>
where
    S: actix::Actor,
    <S as actix::Actor>::Context: ToEnvelope<S, subscription_messages::GetBatch>,
    S: actix::Handler<subscription_messages::GetBatch>,
{
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.subscription_addr
            .do_send(subscription_messages::GetBatch);
    }
}

// Docs for async /actix/fut/future/trait.ActorFuture.html
impl<S> actix::Handler<messages::GetBatchReply> for Actor<S>
where
    S: actix::Actor,
    <S as actix::Actor>::Context: ToEnvelope<S, subscription_messages::GetBatch>,
    S: actix::Handler<subscription_messages::GetBatch>,
{
    // type Result = ResponseActFuture<Self, ()>;
    type Result = ();

    fn handle(
        &mut self,
        get_batch_reply: messages::GetBatchReply,
        _ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
        let data = serde_json::to_value(get_batch_reply.batch.clone()).expect("here");
        self.telemetry
            .record_data(telemetry::PRE_FETCH_QUEUED, data);
        // // Send Get Batch
        // let get_batch_response = self.subscription_addr.send(subscription_messages::GetBatch);
        // Box::pin(
        //     async { get_batch_response.await.expect("this to work") }
        //         .into_actor(self) // converts future to ActorFuture
        //         .map(|_res, _act, _ctx| {
        //             // Do some computation with actor's state or context
        //             Ok(())
        //         }),
        // )
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
    // use std::time::Duration;

    use serde_json::Value;

    use super::*;
    use crate::consumer;
    use crate::controls;
    use crate::telemetry::sink;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[actix::test]
    async fn should_call_subscription_get_batch_on_start() {
        init();

        // Arrange
        let mut subscription =
            controls::consumer::subscription_substitute::SubscriptionSubstitute::new();
        let telemetry = subscription.telemetry();
        let sink = sink::Sink::new();
        telemetry.register(sink.clone());

        let subscription_addr = subscription.start();

        // Act
        let addr = Actor::new(subscription_addr.clone()).start();

        addr.send(messages::Stop).await.expect("send to work");
        subscription_addr
            .send(subscription_messages::Stop)
            .await
            .expect("send to work");

        // Assert
        let batch_called = 1;
        if let Value::Number(count) =
            sink.data_recorded(consumer::subscription::telemetry::GET_BATCH)
        {
            assert_eq!(count.as_i64().unwrap_or(0), batch_called);
        } else {
            panic!("No record of get batch recorded");
        }
    }

    #[actix::test]
    async fn should_assign_subscription_batch_to_pre_fetch_queue() {
        init();

        // Arrange
        let subscription =
            controls::consumer::subscription_substitute::SubscriptionSubstitute::new();
        let subscription_addr = subscription.start();
        let batch = controls::consumer::subscription_substitute::batch();

        // Act
        let mut actor = Actor::new(subscription_addr.clone());
        let telemetry = actor.telemetry();
        let sink = sink::Sink::new();
        telemetry.register(sink.clone());

        let actor_addr = actor.start();
        actor_addr
            .send(messages::GetBatchReply {
                batch: batch.clone(),
            })
            .await
            .expect("send to work");

        subscription_addr.do_send(subscription_messages::Stop);

        // Assert
        let pre_fetched: Vec<()> =
            serde_json::from_value(sink.data_recorded(telemetry::PRE_FETCH_QUEUED))
                .expect("pre_fetch batch to parse from telemetry");
        assert!(pre_fetched == batch);
    }
}
