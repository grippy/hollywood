use chrono::{DateTime, Utc};
use hollywood::{self, async_trait, Actor, Result, RunOpts};
use log::{error, info};
use pretty_env_logger;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};
use types::{ActorXMsg, ActorYMsg, ACTOR_X, ACTOR_Y};

fn now() -> String {
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    now.to_rfc3339()
}

/// ActorX
struct ActorX<'a> {
    _favorite_movie: &'a str,
    hollywood: hollywood::Client,
}

impl<'a> ActorX<'a> {
    fn new(hollywood_client: hollywood::Client) -> Self {
        Self {
            _favorite_movie: "My Life as a Dog",
            hollywood: hollywood_client,
        }
    }
}

#[async_trait]
impl<'a> Actor for ActorX<'a> {
    type Msg = ActorXMsg;

    fn name(&self) -> &'static str {
        ACTOR_X
    }

    async fn request(&mut self, msg: Self::Msg) -> Result<Option<Self::Msg>> {
        match msg {
            ActorXMsg::HelloRequest => {
                info!("hello request");

                // try to ping ActorY here...
                let msg = ActorYMsg::PingRequest { timestamp: now() };
                match self.hollywood.request::<ActorYMsg>(ACTOR_Y, msg).await {
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

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let hollywood_client = hollywood::Client::from_env().await?;
    let actor = ActorX::new(hollywood_client);
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(100u32));
    hollywood::run(opts).await
}
