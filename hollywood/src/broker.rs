use crate::actor::{
    ActorMsg, ActorRequest, ActorSend, ActorSender, ActorSubscribe, HollywoodMsg, SubscribeType,
};
use anyhow::Result;
use log::{error, info, warn};
use nats::asynk::Connection;
use tokio::time::{sleep, Duration};

pub(crate) struct Broker {
    actor_name: String,
    mailbox_names: Vec<String>,
    mailbox_sender: ActorSender,
    mailbox_max_size: Option<u32>,
    nats: Connection,
    subscribe_type: SubscribeType,
}

impl Broker {
    pub(crate) fn new(
        actor_name: String,
        mailbox_names: Vec<String>,
        mailbox_sender: ActorSender,
        mailbox_max_size: Option<u32>,
        nats: Connection,
        subscribe_type: SubscribeType,
    ) -> Self {
        Self {
            actor_name: actor_name,
            mailbox_names: mailbox_names,
            mailbox_sender: mailbox_sender,
            mailbox_max_size: mailbox_max_size,
            nats: nats,
            subscribe_type: subscribe_type,
        }
    }

    async fn spawn(
        actor_name: String,
        mailbox_max_size: Option<u32>,
        mailbox_name: String,
        mailbox_sender: ActorSender,
        nats: Connection,
        subscribe_type: SubscribeType,
    ) -> Result<()> {
        let source = match subscribe_type {
            SubscribeType::Queue => {
                info!(
                    "{} agent subscribing to queue subject {:?}",
                    &actor_name, &mailbox_name
                );
                nats
                    // use the mailbox name as the group
                    .queue_subscribe(&mailbox_name, &mailbox_name)
                    .await?
            }
            SubscribeType::Publish { subject } => {
                info!(
                    "{} agent subscribing to pubsub subject {:?}",
                    &actor_name, &subject
                );
                nats.subscribe(&subject).await?
            }
        };

        let max_size = if mailbox_max_size.is_some() {
            mailbox_max_size.unwrap()
        } else {
            0
        };
        let mut backoff = 0;
        loop {
            // slow down nats reading if mailbox is full
            loop {
                if max_size > 0 && mailbox_sender.len() > max_size as usize {
                    sleep(Duration::from_millis(100)).await;
                } else {
                    break;
                }
            }

            if let Some(nats_msg) = source.try_next() {
                // deserialize nats_msg.data here
                let hollywood_msg: HollywoodMsg = match serde_json::from_slice(&nats_msg.data) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("deserializing nats msg to HollywoodMsg: {:?}", &err);
                        continue;
                    }
                };

                // We should only have request/send here
                // HollywoodMsg::Response type is only
                let (msg_id, msg_version, msg) = match hollywood_msg {
                    HollywoodMsg::Request(resp) => (resp.id, resp.msg_version, resp.msg),
                    HollywoodMsg::Send(send) => (send.id, send.msg_version, send.msg),
                    HollywoodMsg::Publish(publish) => {
                        (publish.id, publish.msg_version, publish.msg)
                    }
                    _ => {
                        todo!("HollywoodMsg: not yet implemented");
                    }
                };

                // create ActorMsg.. if nats msg
                // has a reply handle then send a nats request
                // so we can route the response back to the caller
                let msg = if nats_msg.reply.is_some() {
                    ActorMsg::Request(ActorRequest {
                        id: msg_id,
                        msg: msg,
                        msg_version: msg_version,
                        reply_id: nats_msg.reply.unwrap(),
                    })
                } else {
                    // send-type: queue or pubsub?
                    match subscribe_type {
                        SubscribeType::Queue => ActorMsg::Send(ActorSend {
                            id: msg_id,
                            msg: msg,
                            msg_version: msg_version,
                        }),
                        _ => ActorMsg::Subscribe(ActorSubscribe {
                            id: msg_id,
                            msg: msg,
                            msg_version: msg_version,
                        }),
                    }
                };

                // send to mailbox
                match mailbox_sender.send(msg).await {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("failed to forward msg to actor mailbox: {:?}", &err);
                    }
                }
                backoff = 0;
            } else {
                // backoff max is 1sec
                backoff = std::cmp::min(backoff + 10, 1000);
            }

            if backoff > 0 {
                sleep(Duration::from_millis(backoff)).await;
            }
        }
        // Ok(())
    }

    pub(crate) async fn run(&self) -> Result<()> {
        // spawn broker for each mailbox
        for mailbox_name in &self.mailbox_names {
            let actor_name = self.actor_name.clone();
            let nats = self.nats.clone();
            let mailbox_name = mailbox_name.to_owned();
            let mailbox_sender = self.mailbox_sender.clone();
            let mailbox_max_size = self.mailbox_max_size.clone();
            let subscribe_type = self.subscribe_type.clone();
            tokio::spawn(async move {
                Broker::spawn(
                    actor_name,
                    mailbox_max_size,
                    mailbox_name,
                    mailbox_sender,
                    nats,
                    subscribe_type,
                )
                .await
            });
        }
        Ok(())
    }
}
