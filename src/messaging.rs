pub mod get;
pub mod postgres;

pub use get::*;

#[derive(Debug, Clone)]
pub struct Message;

pub trait Handler {}
