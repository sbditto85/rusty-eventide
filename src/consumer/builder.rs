use actix::{Actor as ActixActor, Addr};
use futures::future::LocalBoxFuture;

use crate::consumer::{actor::Actor, subscription::Subscription, Consumer};

pub struct ConsumerBuilder<F> {
    probe: Option<F>,
}

impl<F> ConsumerBuilder<F> {
    pub fn new() -> Self {
        ConsumerBuilder { probe: None }
    }
}

impl<F> ConsumerBuilder<F>
where
    F: for<'a> FnMut(
        &'a Consumer,
        &'a Addr<Actor>,
        &'a Addr<Subscription>,
    ) -> LocalBoxFuture<'a, ()>,
{
    pub fn set_probe(&mut self, probe: F) {
        self.probe = Some(probe);
    }

    pub async fn start<C>(mut self, category: C) -> ()
    where
        C: Into<String>,
    {
        let consumer = Consumer {
            category: category.into(),
        };

        let actor_address = Actor.start();
        let subscription_address = Subscription.start();

        if let Some(probe) = self.probe.as_mut() {
            probe(&consumer, &actor_address, &subscription_address).await;
        }
    }
}
