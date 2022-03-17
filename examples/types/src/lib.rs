use serde::{Deserialize, Serialize};

// ActorX
pub static ACTOR_X: &'static str = "ActorX";
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ActorXMsg {
    HelloRequest,
    HelloResponse,
    SomeSend,
    Sleep { secs: u64 },
}

// ActorY
pub static ACTOR_Y: &'static str = "ActorY";
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ActorYMsg {
    PingRequest { timestamp: String },
    PingResponse { timestamp: String },
    SomeSend,
}

// ActorZ
pub static ACTOR_Z: &'static str = "ActorZ";
// ActorZZ
pub static ACTOR_ZZ: &'static str = "ActorZZ";

// Pubsub Msg & Subjects
pub static PUBSUB_SUBJECT_ONE: &'static str = "subject-one";
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SubjectOneMsg {
    Event,
}
