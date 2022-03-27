use crate::types::msg::{ActorXMsg, ActorYMsg};
use crate::types::version::V1_0;
use chrono::{DateTime, Utc};
use hollywood::{
    self, async_trait, Actor, Dispatch, DispatchResponse, DispatchType, Handle, Msg, Result,
};
use hollywood_macro::Hollywood;
use log::{debug, error, info};
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

///
///
/// ActorX
///
///

fn now() -> String {
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    now.to_rfc3339()
}

#[derive(Hollywood)]
#[dispatch("ActorXMsg")]
pub struct ActorX {
    actor_y: hollywood::mailbox::Mailbox,
}

impl ActorX {
    pub fn new(actor_y: hollywood::mailbox::Mailbox) -> Self {
        Self { actor_y }
    }
}

impl Actor for ActorX {
    const VERSION: &'static str = V1_0;
}

#[async_trait]
impl Handle<ActorXMsg> for ActorX {
    type Msg = ActorXMsg;

    async fn request(&mut self, msg: Self::Msg) -> Result<Option<Self::Msg>> {
        match msg {
            ActorXMsg::HelloRequest => {
                info!("hello request");
                // try to ping ActorY here...
                let msg = ActorYMsg::PingRequest { timestamp: now() };
                match self.actor_y.request::<ActorYMsg>(msg).await {
                    Ok(msg) => {
                        info!("ActorYMsg::PingRequest response msg: {:?}", &msg);
                    }
                    Err(err) => {
                        error!("ActorYMsg::PingRequest response err: {:?}", &err);
                    }
                }
                Ok(Some(ActorXMsg::HelloResponse))
            }
            ActorXMsg::Sleep { secs } => {
                sleep(Duration::from_secs(secs)).await;
                Ok(Some(ActorXMsg::HelloResponse))
            }
            _ => Ok(None),
        }
    }

    async fn send(&mut self, msg: Self::Msg) -> Result<()> {
        match msg {
            ActorXMsg::HelloRequest => {
                info!("hello send");
            }
            _ => {}
        }
        Ok(())
    }

    // default implementation since this actor subscribes
    // to a queue...
    async fn subscribe(&mut self, _: Self::Msg) -> Result<()> {
        Ok(())
    }
}
