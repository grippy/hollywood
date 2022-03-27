use hollywood::Msg;
use serde::{Deserialize, Serialize};

use crate::types::version;

// // ActorZ
// pub static ACTOR_Z: &'static str = "ActorZ";
// // ActorZZ
// pub static ACTOR_ZZ: &'static str = "ActorZZ";

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ActorXMsg {
    HelloRequest,
    HelloResponse,
    SomeSend,
    Sleep { secs: u64 },
}
impl Msg for ActorXMsg {
    type Type = Self;
    const VERSION: &'static str = version::V1_0;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ActorYMsg {
    PingRequest { timestamp: String },
    PingResponse { timestamp: String },
    SomeSend,
}
impl Msg for ActorYMsg {
    type Type = Self;
    const VERSION: &'static str = version::V1_0;
}

// Pubsub Msg & Subjects
pub static PUBSUB_SUBJECT_ONE: &'static str = "subject-one";
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SubjectOneMsg {
    Event,
}
impl Msg for SubjectOneMsg {
    type Type = Self;
    const VERSION: &'static str = version::V1_0;
}
