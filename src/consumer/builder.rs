use std::pin::Pin;

use actix::Addr;
use futures::future::LocalBoxFuture;

use crate::consumer::{actor::Actor, subscription::Subscription, Consumer};

pub struct ConsumerBuilder;

impl ConsumerBuilder {
    pub fn set_probe<'a, F>(&mut self, probe: F)
    where
        F: FnMut(&'a Consumer, &'a Addr<Actor>, &'a Addr<Subscription>) -> LocalBoxFuture<'a, ()>,
    {
    }

    pub fn start<C>(self, category: C) -> ()
    where
        C: Into<String>,
    {
        //TODO: do this
    }
}
