use crate::actor::{mailbox_name, HollywoodMsg, HollywoodPublish, HollywoodRequest, HollywoodSend};
use crate::common::new_id_as_string;
use crate::env::{hollywood_system, hollywood_system_nats_uri};
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use nats;
use nats::asynk::{Connection, Message};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::time::{sleep, Duration};

/// Initialize nats connection
async fn init_client(nats_uri: String) -> Result<Connection> {
    let mut nats_client = nats::asynk::connect(&nats_uri).await;
    loop {
        match nats_client {
            Err(err) => {
                warn!("error connecting hollywood client to nats: #{:?}", &err);
                sleep(Duration::from_millis(1000)).await;
                nats_client = nats::asynk::connect(&nats_uri).await;
            }
            Ok(_) => {
                info!("connected nats");
                break;
            }
        }
    }
    Ok(nats_client?)
}

/// Hollywood Client for a given system
pub struct Client {
    system_name: String,
    nats: Connection,
}

impl Client {
    pub async fn new(system_name: String, nats_uri: String) -> Result<Self> {
        let nats_client = match init_client(nats_uri).await {
            Ok(nc) => nc,
            Err(err) => return Err(err.into()),
        };

        Ok(Client {
            system_name: system_name,
            nats: nats_client,
        })
    }

    pub async fn from_env() -> Result<Self> {
        let system_name = hollywood_system()?;
        let nats_uri = hollywood_system_nats_uri(system_name.clone())?;
        info!(
            "from_env system:{:?}, nats_uri:{:?}",
            &system_name, &nats_uri
        );
        let nats_client = match init_client(nats_uri).await {
            Ok(nc) => nc,
            Err(err) => return Err(err.into()),
        };
        Ok(Client {
            system_name: system_name,
            nats: nats_client,
        })
    }

    fn serialize<T: Serialize>(msg: &T) -> Result<Vec<u8>> {
        match serde_json::to_vec(msg) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(err.into()),
        }
    }

    fn deserialize<R: DeserializeOwned>(msg: &Vec<u8>) -> Result<R> {
        match serde_json::from_slice(msg) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn publish<M: Serialize>(&self, subject: &str, msg: M) -> Result<()> {
        let msg = Self::serialize(&msg)?;
        let publish = HollywoodPublish {
            id: new_id_as_string(),
            msg: msg,
        };
        let hollywood_msg = HollywoodMsg::Publish(publish);
        let msg = Self::serialize(&hollywood_msg)?;
        debug!(
            "hollywood::publish to subject: {} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        match self.nats.publish(&subject[..], msg).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn send<M: Serialize>(&self, actor_name: &str, msg: M) -> Result<()> {
        let msg = Self::serialize(&msg)?;
        let send = HollywoodSend {
            id: new_id_as_string(),
            msg: msg,
        };
        let hollywood_msg = HollywoodMsg::Send(send);
        let msg = Self::serialize(&hollywood_msg)?;
        let subject = mailbox_name(&self.system_name, &actor_name.to_string());
        debug!(
            "hollywood::send to actor: {} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        match self.nats.publish(&subject[..], msg).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    async fn handle_request<M: DeserializeOwned>(
        &self,
        result: std::io::Result<Message>,
    ) -> Result<M> {
        match result {
            Ok(msg) => {
                let hollywood_msg = match Self::deserialize::<HollywoodMsg>(&msg.data) {
                    Ok(msg) => msg,
                    Err(err) => return Err(err.into()),
                };
                match hollywood_msg {
                    HollywoodMsg::Response(resp) => {
                        // we should only have one or the other here
                        // with a value of some kind
                        // msg = Option<Vec<u8>>
                        if resp.msg.is_some() {
                            let msg = resp.msg.unwrap();
                            match Self::deserialize::<M>(&msg) {
                                Ok(msg) => return Ok(msg),
                                Err(err) => return Err(err.into()),
                            }
                        } else {
                            // error = Option<String>
                            if resp.error.is_some() {
                                let err = resp.error.unwrap();
                                return Err(anyhow!(
                                    "msg id {} response err: {:?}",
                                    &resp.id,
                                    &err
                                ));
                            }
                        }
                        Err(anyhow!("not sure what happened"))
                    }
                    _ => {
                        // we should only have Response type here
                        Err(anyhow!("request received a non HollywoodMsg::Response"))
                    }
                }
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn request_timeout<M: Serialize + DeserializeOwned>(
        &self,
        actor_name: &str,
        msg: M,
        timeout_secs: u64,
    ) -> Result<M> {
        let msg = Self::serialize(&msg)?;
        let req = HollywoodRequest {
            id: new_id_as_string(),
            msg: msg,
        };
        let hollywood_msg = HollywoodMsg::Request(req);
        let msg = Self::serialize(&hollywood_msg)?;
        let subject = mailbox_name(&self.system_name, &actor_name.to_string());
        let timeout = std::time::Duration::from_secs(timeout_secs);
        debug!(
            "hollywood::request_timeout to actor:{} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        let result = self.nats.request_timeout(&subject[..], msg, timeout).await;
        self.handle_request::<M>(result).await
    }

    pub async fn request<M: Serialize + DeserializeOwned>(
        &self,
        actor_name: &str,
        msg: M,
    ) -> Result<M> {
        let msg = Self::serialize(&msg)?;
        let req = HollywoodRequest {
            id: new_id_as_string(),
            msg: msg,
        };
        let hollywood_msg = HollywoodMsg::Request(req);
        let msg = Self::serialize(&hollywood_msg)?;
        let subject = mailbox_name(&self.system_name, &actor_name.to_string());
        debug!(
            "hollywood::request to actor:{} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        let result = self.nats.request(&subject[..], msg).await;
        self.handle_request::<M>(result).await
    }
}
