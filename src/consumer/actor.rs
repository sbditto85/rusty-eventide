use actix::ActorContext;

pub mod messages;

pub struct Actor;

impl actix::Actor for Actor {
    type Context = actix::Context<Self>;
}

impl actix::Handler<messages::Stop> for Actor {
    type Result = ();

    fn handle(&mut self, _msg: messages::Stop, ctx: &mut actix::Context<Self>) {
        ctx.stop()
    }
}
