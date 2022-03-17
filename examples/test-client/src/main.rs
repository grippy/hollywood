use hollywood::Result;
use log::{error, info};
use pretty_env_logger;
use tokio::time::{sleep, Duration};
use types::{ActorXMsg, ActorYMsg, SubjectOneMsg, ACTOR_X, ACTOR_Y, PUBSUB_SUBJECT_ONE};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let system_name = "examples".into();
    let nats_uri = "nats://127.0.0.1:14222".into();
    let hollywood = hollywood::Client::new(system_name, nats_uri).await?;
    loop {
        // ActorX send
        match hollywood
            .send::<ActorXMsg>(ACTOR_X, ActorXMsg::HelloRequest)
            .await
        {
            Ok(msg) => {
                info!("ActorXMsg::HelloRequest send resp: {:?}", &msg);
            }
            Err(err) => {
                error!("ActorXMsg::HelloRequest send err: {:?}", &err);
            }
        }

        // ActorX request
        match hollywood
            .request::<ActorXMsg>(ACTOR_X, ActorXMsg::HelloRequest)
            .await
        {
            Ok(msg) => {
                info!("ActorXMsg::HelloRequest response msg: {:?}", &msg);
            }
            Err(err) => {
                error!("ActorXMsg::HelloRequest response err: {:?}", &err);
            }
        }

        // ActorX request with timeout success
        match hollywood
            .request_timeout::<ActorXMsg>(ACTOR_X, ActorXMsg::Sleep { secs: 1 }, 2)
            .await
        {
            Ok(msg) => {
                info!("with timeout ActorXMsg::Sleep response msg: {:?}", &msg);
            }
            Err(err) => {
                error!("with timeout ActorXMsg::Sleep response err: {:?}", &err);
            }
        }

        // ActorX request with timeout error
        match hollywood
            .request_timeout::<ActorXMsg>(ACTOR_X, ActorXMsg::Sleep { secs: 2 }, 1)
            .await
        {
            Ok(msg) => {
                info!("with timeout ActorXMsg::Sleep response msg: {:?}", &msg);
            }
            Err(err) => {
                error!("with timeout ActorXMsg::Sleep response err: {:?}", &err);
            }
        }

        // ActorY send
        match hollywood
            .send::<ActorYMsg>(ACTOR_Y, ActorYMsg::SomeSend)
            .await
        {
            Ok(msg) => {
                info!("ActorYMsg::SomeSend resp: {:?}", &msg);
            }
            Err(err) => {
                error!("ActorYMsg::SomeSend err: {:?}", &err);
            }
        }

        // Publish subject message
        match hollywood
            .publish::<SubjectOneMsg>(PUBSUB_SUBJECT_ONE, SubjectOneMsg::Event)
            .await
        {
            Ok(msg) => {
                info!("SubjectOneMsg::Event resp: {:?}", &msg);
            }
            Err(err) => {
                error!("SubjectOneMsg::Event err: {:?}", &err);
            }
        }

        sleep(Duration::from_millis(3000)).await;
    }
}
