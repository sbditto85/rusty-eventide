use std::error::Error as StdError;

use thiserror::Error;

pub mod get;
pub mod postgres;

pub use get::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message;

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Error in handler code {0}")]
    HandlerError(#[from] Box<dyn StdError>),
}

pub trait Handler {
    fn handle(&mut self, message: Message) -> Result<(), HandleError>;
}
