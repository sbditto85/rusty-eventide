use std::error::Error as StdError;

use thiserror::Error;

pub mod get;
pub mod postgres;

pub use get::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageData {
    pub global_position: u64,
}

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Error in handler code {0}")]
    HandlerError(#[from] Box<dyn StdError + Send>),
    #[error("Unable to get messages {0}")]
    GetError(#[from] GetError),
    #[error("Missing Handler")]
    MissingHandler,
}

impl<E: StdError + Send + 'static> From<Box<E>> for HandleError {
    fn from(error: Box<E>) -> Self {
        HandleError::HandlerError(error)
    }
}

pub trait Handler: std::fmt::Debug {
    fn handle(&mut self, message: MessageData) -> Result<(), HandleError>;
}
