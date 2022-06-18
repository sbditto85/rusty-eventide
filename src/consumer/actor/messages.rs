#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct GetBatchReply {
    pub batch: Vec<()>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Stop;
