use hollywood::{self, Result, RunOpts};
use pretty_env_logger;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let actor = system::ActorZ::new();
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(100u32));
    hollywood::run(opts).await
}
