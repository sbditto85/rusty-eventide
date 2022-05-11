pub mod messages;

pub struct Subscription;

impl actix::Actor for Subscription {
    type Context = actix::Context<Self>;
}

impl actix::Handler<messages::Stop> for Subscription {
    type Result = ();

    fn handle(&mut self, msg: messages::Stop, ctx: &mut actix::Context<Self>) {}
}
