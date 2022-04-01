use hollywood::{self, Result, RunOpts};
use log::{info, warn};
use pretty_env_logger;
use redis;
use tokio::time::{sleep, Duration};

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

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let redis_uri = "redis://127.0.0.1/";
    let redis_conn = init_redis_client(redis_uri).await.unwrap();
    let actor = system::ActorY::new(redis_conn);
    let opts = RunOpts::from_env(actor)?.with_actor_mailbox_max_size(Some(5u32));
    hollywood::run(opts).await
}
