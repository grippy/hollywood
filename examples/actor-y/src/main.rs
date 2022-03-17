use hollywood::{async_trait, Actor, Result, RunOpts};
use log::{info, warn};
use pretty_env_logger;
use redis;
// use futures::prelude::*;
use redis::AsyncCommands;
use tokio::time::{sleep, Duration};
use types::{ActorYMsg, ACTOR_Y};

// init_redis_client returns an async redis connection
// TODO: should define a timeout or max retries
async fn init_redis_client(redis_uri: &str) -> Result<redis::aio::Connection> {
    let mut result = redis::Client::open(redis_uri);
    loop {
        match result {
            Err(err) => {
                warn!("error creating redis client: #{:?}", &err);
                sleep(Duration::from_millis(1000)).await;
                result = redis::Client::open(redis_uri);
            }
            Ok(_) => {
                info!("redis client initialized...");
                break;
            }
        }
    }
    let client = result.unwrap();
    let mut result = client.get_async_connection().await;
    loop {
        match result {
            Err(err) => {
                warn!("error creating redis client: #{:?}", &err);
                sleep(Duration::from_millis(1000)).await;
                result = client.get_async_connection().await;
            }
            Ok(_) => {
                info!("redis connection created...");
                break;
            }
        };
    }
    let conn = result.unwrap();
    Ok(conn)
}

/// ActorY Example
struct ActorY {
    redis: redis::aio::Connection,
}

impl ActorY {
    fn new(redis_conn: redis::aio::Connection) -> Self {
        Self { redis: redis_conn }
    }
}

#[async_trait]
impl Actor for ActorY {
    type Msg = ActorYMsg;

    fn name(&self) -> &'static str {
        ACTOR_Y
    }

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

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let redis_uri = "redis://127.0.0.1/";
    let redis_conn = init_redis_client(redis_uri).await.unwrap();
    let actor = ActorY::new(redis_conn);
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(5u32));
    hollywood::run(opts).await
}
