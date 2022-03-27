use hollywood::{self, Result, RunOpts};
use pretty_env_logger;
use system::actor::actor_zz::ActorZZ;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let actor = ActorZZ::new();
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(100u32));
    hollywood::run(opts).await
}
