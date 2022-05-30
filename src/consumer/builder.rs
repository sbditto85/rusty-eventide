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

#[cfg(test)]
mod unit_tests {
    use futures::future::{FutureExt, LocalBoxFuture};

    use crate::consumer::{actor, subscription};

    use super::*;
    use crate::controls;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /////////////////////
    // Start (Using Builder to emulate their class methods)
    /////////////////////

    #[actix::test]
    async fn should_assign_category_on_start() {
        init();

        // Arrange
        let mut consumer_category = None;
        let mut builder = controls::consumer::builder::example();
        builder.set_probe(
            |consumer, actor_address, subscription_address| -> LocalBoxFuture<()> {
                consumer_category = Some(consumer.category.clone());

                async {
                    actor_address
                        .send(actor::messages::Stop)
                        .await
                        .expect("Actor stop to work");
                    subscription_address
                        .send(subscription::messages::Stop)
                        .await
                        .expect("Subscription stop to work");
                }
                .boxed_local()
            },
        );
        let category = controls::messages::category::example();

        // Act
        builder.start(&category).await;

        // Assert
        assert_eq!(consumer_category, Some(category));
    }
}
