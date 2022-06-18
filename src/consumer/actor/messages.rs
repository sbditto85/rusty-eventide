use std::collections::VecDeque;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Dispatch;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct GetBatchReply {
    pub batch: VecDeque<()>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Stop;
