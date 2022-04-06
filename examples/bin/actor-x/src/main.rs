use hollywood::{self, ActorMailbox, Result, RunOpts};
use system::ActorX;
use system::ActorY;
use system::ActorYMsg;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let actor_y = ActorY::mailbox_from_env::<ActorYMsg>().await?;
    let actor = ActorX::new(actor_y);
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(100u32));
    hollywood::run(opts).await
}
