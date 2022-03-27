use hollywood::{ActorMailbox, Result};
use log::{error, info};
use pretty_env_logger;

use system::actor::actor_x::ActorX;
use system::actor::actor_y::ActorY;
use system::actor::actor_z::ActorZ;
use system::types::msg::{ActorXMsg, ActorYMsg, SubjectOneMsg, PUBSUB_SUBJECT_ONE};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let system_name: String = "examples".into();
    let nats_uri: String = "nats://127.0.0.1:14222".into();

    // let actor_x = ActorX::mailbox::<ActorXMsg>(system_name.clone(), nats_uri.clone()).await?;
    loop {
        //     // ActorX send
        //     info!("request ActorXMsg::HelloRequest");
        //     match actor_x.send::<ActorXMsg>(ActorXMsg::HelloRequest).await {
        //         Ok(msg) => {
        //             info!("ActorXMsg::HelloRequest send resp: {:?}", &msg);
        //         }
        //         Err(err) => {
        //             error!("ActorXMsg::HelloRequest send err: {:?}", &err);
        //         }
        //     }

        //     // ActorX request
        //     match actor_x.request::<ActorXMsg>(ActorXMsg::HelloRequest).await {
        //         Ok(msg) => {
        //             info!("ActorXMsg::HelloRequest response msg: {:?}", &msg);
        //         }
        //         Err(err) => {
        //             error!("ActorXMsg::HelloRequest response err: {:?}", &err);
        //         }
        //     }

        //     // ActorX request with timeout success
        //     match actor_x
        //         .request_timeout::<ActorXMsg>(ActorXMsg::Sleep { secs: 1 }, 2)
        //         .await
        //     {
        //         Ok(msg) => {
        //             info!("with timeout ActorXMsg::Sleep response msg: {:?}", &msg);
        //         }
        //         Err(err) => {
        //             error!("with timeout ActorXMsg::Sleep response err: {:?}", &err);
        //         }
        //     }

        //     // ActorX request with timeout error
        //     match actor_x
        //         .request_timeout::<ActorXMsg>(ActorXMsg::Sleep { secs: 2 }, 1)
        //         .await
        //     {
        //         Ok(msg) => {
        //             info!("with timeout ActorXMsg::Sleep response msg: {:?}", &msg);
        //         }
        //         Err(err) => {
        //             error!("with timeout ActorXMsg::Sleep response err: {:?}", &err);
        //         }
        //     }

        // ActorY send
        let actor_y = ActorX::mailbox::<ActorYMsg>(system_name.clone(), nats_uri.clone()).await?;
        match actor_y.send::<ActorYMsg>(ActorYMsg::SomeSend).await {
            Ok(msg) => {
                info!("ActorYMsg::SomeSend resp: {:?}", &msg);
            }
            Err(err) => {
                error!("ActorYMsg::SomeSend err: {:?}", &err);
            }
        }

        // // Publish subject message
        // let actor_z =
        //     ActorZ::mailbox::<SubjectOneMsg>(system_name.clone(), nats_uri.clone()).await?;

        // match actor_z.publish::<SubjectOneMsg>(SubjectOneMsg::Event).await {
        //     Ok(msg) => {
        //         info!("SubjectOneMsg::Event resp: {:?}", &msg);
        //     }
        //     Err(err) => {
        //         error!("SubjectOneMsg::Event err: {:?}", &err);
        //     }
        // }

        sleep(Duration::from_millis(3000)).await;
    }

    // Ok(())
}
