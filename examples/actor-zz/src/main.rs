use hollywood::{self, async_trait, Actor, Result, RunOpts, SubscribeType};
use log::info;
use pretty_env_logger;
use types::{SubjectOneMsg, ACTOR_ZZ, PUBSUB_SUBJECT_ONE};

/// ActorZZ
struct ActorZZ {}

impl ActorZZ {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Actor for ActorZZ {
    type Msg = SubjectOneMsg;

    fn name(&self) -> &'static str {
        ACTOR_ZZ
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
    // to a a pubsub topic...
    async fn subscribe(&mut self, msg: Self::Msg) -> Result<()> {
        match msg {
            SubjectOneMsg::Event => {
                info!("subscribe event actor-zz");
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let actor = ActorZZ::new();
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(100u32));
    hollywood::run(opts).await
}
