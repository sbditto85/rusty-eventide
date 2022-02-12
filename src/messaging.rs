pub mod get;
pub mod postgres;

pub use get::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message;

pub trait Handler {
    fn handle(&mut self, message: Message);
}
