#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct GetBatch;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Stop;
