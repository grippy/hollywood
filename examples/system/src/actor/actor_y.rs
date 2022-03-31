use crate::types::msg::ActorYMsg;
use crate::types::version::V1_0;
use hollywood::prelude::actor::*;
use hollywood_macro::Hollywood;
use log::{debug, info};
use redis;
use redis::AsyncCommands;

/// ActorY
#[derive(Hollywood)]
#[dispatch(ActorYMsg)]
pub struct ActorY {
    redis: redis::aio::Connection,
}

impl ActorY {
    pub fn new(redis_conn: redis::aio::Connection) -> Self {
        Self { redis: redis_conn }
    }
}

impl Actor for ActorY {
    const VERSION: &'static str = V1_0;
}

#[async_trait]
impl Handle<ActorYMsg> for ActorY {
    type Msg = ActorYMsg;

    async fn request(&mut self, msg: Self::Msg) -> Result<Option<Self::Msg>> {
        match msg {
            ActorYMsg::PingRequest { timestamp } => {
                info!("PingRequest {}", timestamp);
                Ok(Some(ActorYMsg::PingResponse { timestamp }))
            }
            _ => Ok(None),
        }
    }

    async fn send(&mut self, msg: Self::Msg) -> Result<()> {
        match msg {
            ActorYMsg::SomeSend => {
                info!("Some send... update redis");
                match self.redis.set::<_, _, String>("some_send", 42).await {
                    Ok(rv) => {
                        info!("some_send, redis reply: {}", rv)
                    }
                    Err(err) => return Err(err.into()),
                }
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
