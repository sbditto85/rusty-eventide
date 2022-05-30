use actix::ActorContext;

pub mod messages;
pub mod telemetry;

pub struct Subscription;

impl actix::Actor for Subscription {
    type Context = actix::Context<Self>;
}

impl actix::Handler<messages::GetBatch> for Subscription {
    type Result = ();

    fn handle(&mut self, _msg: messages::GetBatch, ctx: &mut actix::Context<Self>) {
        // ctx.stop()
    }
}

impl actix::Handler<messages::Stop> for Subscription {
    type Result = ();

    fn handle(&mut self, _msg: messages::Stop, ctx: &mut actix::Context<Self>) {
        ctx.stop()
    }
}
