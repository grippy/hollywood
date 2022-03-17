use hollywood::{self, async_trait, Actor, Result, RunOpts, SubscribeType};
use log::info;
use pretty_env_logger;
use types::{SubjectOneMsg, ACTOR_Z, PUBSUB_SUBJECT_ONE};

/// ActorZ
struct ActorZ {}

impl ActorZ {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Actor for ActorZ {
    type Msg = SubjectOneMsg;

    fn name(&self) -> &'static str {
        ACTOR_Z
    }

    fn subscribe_type(&self) -> SubscribeType {
        SubscribeType::Publish {
            subject: PUBSUB_SUBJECT_ONE,
        }
    }

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
                info!("subscribe event actor-z");
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let actor = ActorZ::new();
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(100u32));
    hollywood::run(opts).await
}
