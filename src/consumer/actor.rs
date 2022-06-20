use std::collections::VecDeque;

use actix::{
    dev::ToEnvelope,
    Actor as ActixActor,
    ActorContext,
    // ActorFutureExt,
    Addr,
    AsyncContext,
    // AsyncContext,
    // ResponseActFuture,
    // WrapFuture,
};

use crate::{
    consumer::{subscription::messages as subscription_messages, Consumer},
    telemetry::Telemetry,
};

pub mod messages;
pub mod telemetry;

pub struct Actor<S: actix::Actor> {
    subscription_addr: Addr<S>,
    consumer: Consumer,
    telemetry: Telemetry,
    pre_fetch_queue: VecDeque<()>,
}

impl<S: actix::Actor> Actor<S> {
    pub fn new(subscription_addr: Addr<S>, consumer: Consumer) -> Self {
        Self {
            subscription_addr,
            consumer,
            telemetry: Telemetry::new(),
            pre_fetch_queue: VecDeque::new(),
        }
    }

    pub fn telemetry(&mut self) -> &mut Telemetry {
        &mut self.telemetry
    }

    pub fn pre_fetch_queue(&self) -> &VecDeque<()> {
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
impl<S> actix::Handler<messages::Dispatch> for Actor<S>
where
    S: actix::Actor,
    <S as actix::Actor>::Context: ToEnvelope<S, subscription_messages::GetBatch>,
    S: actix::Handler<subscription_messages::GetBatch>,
{
    // type Result = ResponseActFuture<Self, ()>;
    type Result = ();

    fn handle(
        &mut self,
        _msg: messages::Dispatch,
        _ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
        self.telemetry.record(telemetry::DISPATCH);
        if let Some(message) = self.pre_fetch_queue.pop_front() {
            self.consumer.dispatch(message);
        }
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
        mut get_batch_reply: messages::GetBatchReply,
        ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
        let data = serde_json::to_value(get_batch_reply.batch.clone()).expect("here");
        self.telemetry
            .record_data(telemetry::PRE_FETCH_QUEUED, data);

        self.pre_fetch_queue.append(&mut get_batch_reply.batch);

        ctx.notify(messages::Dispatch);

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

        let consumer = controls::consumer::example();

        // Act
        let addr = Actor::new(subscription_addr.clone(), consumer).start();

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

        let consumer = controls::consumer::example();

        // Act
        let mut actor = Actor::new(subscription_addr.clone(), consumer);
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
        let pre_fetched: VecDeque<()> =
            serde_json::from_value(sink.data_recorded(telemetry::PRE_FETCH_QUEUED))
                .expect("pre_fetch batch to parse from telemetry");
        assert!(pre_fetched == batch);
    }

    #[actix::test]
    async fn should_send_dispatch_when_sent_subscription_batch_and_pre_fetch_queue_empty() {
        init();

        // Arrange
        let subscription =
            controls::consumer::subscription_substitute::SubscriptionSubstitute::new();
        let subscription_addr = subscription.start();
        let batch = controls::consumer::subscription_substitute::batch();

        let consumer = controls::consumer::example();

        // Act
        let mut actor = Actor::new(subscription_addr.clone(), consumer);
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
        actor_addr.do_send(messages::Stop);

        // Assert
        assert!(sink.recorded(telemetry::DISPATCH));
    }

    #[actix::test]
    #[ignore]
    async fn should_not_send_dispatch_when_sent_subscription_batch_and_pre_fetch_queue_not_empty() {
        init();
    }

    #[actix::test]
    async fn should_call_consumer_on_dispatch() {
        init();

        // Arrange
        let subscription =
            controls::consumer::subscription_substitute::SubscriptionSubstitute::new();
        let subscription_addr = subscription.start();

        let mut consumer = controls::consumer::example();
        let telemetry = consumer.telemetry();
        let sink = sink::Sink::new();
        telemetry.register(sink.clone());

        let batch = controls::consumer::subscription_substitute::batch();

        let mut actor = Actor::new(subscription_addr.clone(), consumer);
        actor.pre_fetch_queue = batch;

        // Act

        let actor_addr = actor.start();
        actor_addr
            .send(messages::Dispatch)
            .await
            .expect("send to work");

        subscription_addr.do_send(subscription_messages::Stop);
        actor_addr.do_send(messages::Stop);

        // Assert
        assert!(sink.recorded(consumer::telemetry::DISPATCH));
    }

    #[actix::test]
    #[ignore]
    async fn should_request_more_on_reply_when_queue_not_at_limit() {
        init();
    }

    #[actix::test]
    #[ignore]
    async fn should_not_request_more_on_reply_when_queue_above_limit() {
        init();
    }

    #[actix::test]
    #[ignore]
    async fn should_request_more_on_dispatch_when_queue_back_down_to_limit() {
        init();
    }

    #[actix::test]
    #[ignore]
    async fn should_call_dispatch_on_dispatch_if_queue_is_not_empty() {
        init();
    }

    #[actix::test]
    #[ignore]
    async fn should_not_call_dispatch_on_dispatch_if_queue_is_empty() {
        init();
    }
}
