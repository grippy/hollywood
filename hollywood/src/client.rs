use crate::actor::{HollywoodMsg, HollywoodPublish, HollywoodRequest, HollywoodSend, Msg};
use crate::common::new_id_as_string;
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use nats;
use nats::asynk::{Connection, Message};
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
    nats: Connection,
}

impl Client {
    pub async fn new(nats_uri: String) -> Result<Self> {
        let nats_client = match init_client(nats_uri).await {
            Ok(nc) => nc,
            Err(err) => return Err(err.into()),
        };

        Ok(Client { nats: nats_client })
    }

    pub async fn publish<M: Msg>(&self, subject: &str, msg: M) -> Result<()> {
        let msg = msg.into_bytes()?;
        let msg_version = M::version();
        let publish = HollywoodPublish {
            id: new_id_as_string(),
            msg: msg,
            msg_version: msg_version.to_owned(),
        };
        let hollywood_msg = HollywoodMsg::Publish(publish);
        let msg = hollywood_msg.into_bytes()?;
        debug!(
            "hollywood::publish to subject: {} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        match self.nats.publish(&subject[..], msg).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn send<M: Msg>(&self, subject: &str, msg: M) -> Result<()> {
        let msg = msg.into_bytes()?;
        let msg_version = M::version();
        let send = HollywoodSend {
            id: new_id_as_string(),
            msg: msg,
            msg_version: msg_version.to_owned(),
        };
        let hollywood_msg = HollywoodMsg::Send(send);
        let msg = hollywood_msg.into_bytes()?;
        debug!(
            "hollywood::send to actor: {} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        match self.nats.publish(&subject[..], msg).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    async fn handle_request<M: Msg>(&self, result: std::io::Result<Message>) -> Result<M> {
        match result {
            Ok(msg) => {
                let hollywood_msg = match HollywoodMsg::from_bytes(&msg.data) {
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
                            match M::from_bytes(&msg) {
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

    pub async fn request_timeout<M: Msg>(
        &self,
        subject: &str,
        msg: M,
        timeout_secs: u64,
    ) -> Result<M> {
        let msg = msg.into_bytes()?;
        let msg_version = M::version();
        let req = HollywoodRequest {
            id: new_id_as_string(),
            msg: msg,
            msg_version: msg_version.to_owned(),
        };
        let hollywood_msg = HollywoodMsg::Request(req);
        let msg = hollywood_msg.into_bytes()?;
        // let subject = mailbox_name(&self.system_name, &actor_name.to_string());
        let timeout = std::time::Duration::from_secs(timeout_secs);
        debug!(
            "hollywood::request_timeout to actor:{} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        let result = self.nats.request_timeout(&subject[..], msg, timeout).await;
        self.handle_request::<M>(result).await
    }

    pub async fn request<M: Msg>(&self, subject: &str, msg: M) -> Result<M> {
        let msg = msg.into_bytes()?;
        let msg_version = M::version();
        let req = HollywoodRequest {
            id: new_id_as_string(),
            msg: msg,
            msg_version: msg_version.to_owned(),
        };
        let hollywood_msg = HollywoodMsg::Request(req);
        let msg = hollywood_msg.into_bytes()?;
        // let subject = mailbox_name(&self.system_name, &actor_name.to_string());
        debug!(
            "hollywood::request to actor:{} w/ msg: {:?}",
            &subject, &hollywood_msg
        );
        let result = self.nats.request(&subject[..], msg).await;
        self.handle_request::<M>(result).await
    }
}

pub mod mailbox {

    use super::{debug, info};
    use crate::{env, Actor, Dispatch, Msg, Result, SubscribeType};
    use std::io::{Error, ErrorKind};

    #[allow(dead_code)]
    pub struct Mailbox {
        system_name: String,
        actor_name: &'static str,
        actor_version: &'static str,
        msg_name: &'static str,
        msg_version: &'static str,
        mailbox_name: String,
        hollywood: super::Client,
    }

    impl Mailbox {
        pub async fn new<A: Actor + Dispatch, M: Msg>(
            system_name: String,
            nats_uri: String,
        ) -> Result<Mailbox> {
            let actor_name = A::type_name();
            let actor_version = A::version();
            let msg_name = M::name();
            let msg_version = M::version();
            let mailbox_name = match A::subscribe_type() {
                SubscribeType::Queue => {
                    // check to see if this msg_name/msg_version
                    // is supported by this Actor::dispatch_types
                    let actor_type = format!("{}/{}", &actor_name, &actor_version);
                    let msg_type = format!("{}/{}", &msg_name, &msg_version);
                    debug!(
                        "{} allowable dispatch types: {:?}",
                        &actor_type,
                        A::dispatch_types().join(",")
                    );
                    A::dispatch_types()
                        .into_iter()
                        .find(|item| item == &msg_type)
                        .ok_or(Error::new(
                            ErrorKind::Unsupported,
                            format!("{} doesn't support message type {}", &actor_type, &msg_type),
                        ))?;
                    let mailbox_name = crate::actor::mailbox_name(
                        &system_name,
                        &format!("{}::{}", &actor_type, &msg_type),
                    );
                    mailbox_name
                }
                SubscribeType::Publish { subject } => subject.to_owned(),
            };
            let hollywood_client = super::Client::new(nats_uri).await?;
            Ok(Mailbox {
                system_name,
                actor_name,
                actor_version,
                msg_name,
                msg_version,
                mailbox_name,
                hollywood: hollywood_client,
            })
        }

        pub async fn from_env<A: Actor + Dispatch, M: Msg>() -> Result<Self> {
            let system_name = env::hollywood_system()?;
            let nats_uri = env::hollywood_system_nats_uri(system_name.clone())?;
            info!(
                "Client from_env system:{:?}, nats_uri:{:?}",
                &system_name, &nats_uri
            );
            Self::new::<A, M>(system_name, nats_uri).await
        }

        fn check_type(&self, msg_name: &'static str, msg_version: &'static str) -> Result<()> {
            if msg_name != self.msg_name || msg_version != self.msg_version {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "actor mailbox expects message/version of {}/{}",
                        &self.msg_name, &self.msg_version
                    ),
                )
                .into());
            }
            Ok(())
        }

        pub async fn request<M: Msg>(&self, msg: M) -> Result<M> {
            self.check_type(M::name(), M::version())?;
            let subject = &self.mailbox_name[..];
            self.hollywood.request(subject, msg).await
        }

        pub async fn request_timeout<M: Msg>(&self, msg: M, timeout_secs: u64) -> Result<M> {
            self.check_type(M::name(), M::version())?;
            let subject = &self.mailbox_name[..];
            self.hollywood
                .request_timeout(subject, msg, timeout_secs)
                .await
        }

        pub async fn send<M: Msg>(&self, msg: M) -> Result<()> {
            self.check_type(M::name(), M::version())?;
            let subject = &self.mailbox_name[..];
            self.hollywood.send(subject, msg).await
        }

        pub async fn publish<M: Msg>(&self, msg: M) -> Result<()> {
            self.check_type(M::name(), M::version())?;
            let subject = &self.mailbox_name[..];
            self.hollywood.publish(subject, msg).await
        }
    }
}
