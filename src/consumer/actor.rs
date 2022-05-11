pub mod messages;

pub struct Actor;

impl actix::Actor for Actor {
    type Context = actix::Context<Self>;
}

impl actix::Handler<messages::Stop> for Actor {
    type Result = ();

    fn handle(&mut self, msg: messages::Stop, ctx: &mut actix::Context<Self>) {}
}
