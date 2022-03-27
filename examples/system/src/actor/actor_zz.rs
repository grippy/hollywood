use crate::types::msg::{SubjectOneMsg, PUBSUB_SUBJECT_ONE};
use crate::types::version;
use hollywood::prelude::actor::*;
use hollywood_macro::Hollywood;
use log::{debug, info};

/// ActorZ
#[derive(Hollywood)]
#[dispatch(SubjectOneMsg)]
pub struct ActorZZ {}

#[allow(dead_code)]
impl ActorZZ {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for ActorZZ {
    const VERSION: &'static str = version::V1_0;

    fn subscribe_type() -> SubscribeType {
        SubscribeType::Publish {
            subject: PUBSUB_SUBJECT_ONE,
        }
    }
}

#[async_trait]
impl Handle<SubjectOneMsg> for ActorZZ {
    type Msg = SubjectOneMsg;

    async fn request(&mut self, _: Self::Msg) -> Result<Option<Self::Msg>> {
        Ok(None)
    }

    async fn send(&mut self, _: Self::Msg) -> Result<()> {
        Ok(())
    }

    // default implementation since this actor subscribes
    // to a queue...
    async fn subscribe(&mut self, msg: Self::Msg) -> Result<()> {
        match msg {
            SubjectOneMsg::Event => {
                info!("subscribe event actor-zz: {:?}", &msg);
            }
        }

        Ok(())
    }
}
