use crate::actor::{
    ActorMsg, ActorRequest, ActorSend, ActorSender, ActorSubscribe, HollywoodMsg, SubscribeType,
};
use crate::common::epoch_as_millis;
use crate::system::{SystemComponent, SystemInbox, SystemMsg, SystemSender};
use anyhow::Result;
use log::{error, info, warn};
use nats::asynk::Connection;
use tokio::time::{sleep, Duration};

pub struct Broker {
    system_inbox: SystemInbox,
    mailbox_names: Vec<String>,
    mailbox_sender: ActorSender,
    mailbox_max_size: Option<u32>,
    nats: Connection,
    subscribe_type: SubscribeType,
}

impl Broker {
    pub fn new(
        system_inbox: SystemInbox,
        mailbox_names: Vec<String>,
        mailbox_sender: ActorSender,
        mailbox_max_size: Option<u32>,
        nats: Connection,
        subscribe_type: SubscribeType,
    ) -> Self {
        Self {
            system_inbox: system_inbox,
            mailbox_names: mailbox_names,
            mailbox_sender: mailbox_sender,
            mailbox_max_size: mailbox_max_size,
            nats: nats,
            subscribe_type: subscribe_type,
        }
    }

    pub fn system_inbox(&self) -> SystemSender {
        self.system_inbox.0.clone()
    }

    /// Spawns a broker thread that reads messages
    /// for a given actor+version msg type+version.
    async fn spawn(
        actor_name: String,
        system_inbox: SystemInbox,
        mailbox_max_size: Option<u32>,
        mailbox_name: String,
        mailbox_sender: ActorSender,
        nats: Connection,
        subscribe_type: SubscribeType,
    ) -> Result<()> {
        let nats_subject = match subscribe_type {
            SubscribeType::Queue => mailbox_name,
            SubscribeType::Publish { subject } => subject.to_owned(),
        };
        let nats_broker = match subscribe_type {
            SubscribeType::Queue => {
                info!(
                    "{} agent broker subscribing to queue subject {:?}",
                    &actor_name, &nats_subject
                );
                nats
                    // use the mailbox name as the group name
                    .queue_subscribe(&nats_subject, &nats_subject)
                    .await?
            }
            SubscribeType::Publish { subject } => {
                info!(
                    "{} agent broker subscribing to pubsub subject {:?}",
                    &actor_name, &nats_subject
                );
                nats.subscribe(&subject).await?
            }
        };

        let system_inbox_recv = system_inbox.1;
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

            // read inbox messages...
            if let Ok(sys_msg) = system_inbox_recv.try_recv() {
                match sys_msg {
                    SystemMsg::Health(reply) => {
                        // should send OK back?
                        let _ = reply
                            .send(SystemMsg::Heartbeat(
                                SystemComponent::Broker,
                                epoch_as_millis(),
                            ))
                            .await;
                    }
                    SystemMsg::Shutdown => {
                        // the broker can stop consuming messages
                        // now...
                        info!(
                            "{} agent broker shutting down... stopping reads for subject {:?}",
                            &actor_name, &nats_subject
                        );
                        break;
                    }
                    _ => {}
                }
            }

            if let Some(nats_msg) = nats_broker.try_next() {
                // deserialize nats_msg.data here
                let hollywood_msg: HollywoodMsg = match serde_json::from_slice(&nats_msg.data) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("broker deserializing nats msg to HollywoodMsg: {:?}", &err);
                        continue;
                    }
                };

                // convert hollywood_msg => actor_msg
                let actor_msg = match hollywood_msg {
                    HollywoodMsg::Request(req) => {
                        // HollywoodMsg::Request should always
                        // have a reply handle...
                        // but since this field is optional,
                        // convert this to a "Send" if it's missing
                        if nats_msg.reply.is_some() {
                            ActorMsg::Request(ActorRequest {
                                id: req.id,
                                msg: req.msg,
                                msg_version: req.msg_version,
                                reply_id: nats_msg.reply.unwrap(),
                            })
                        } else {
                            warn!(
                                "broker received HollywoodMsg::Request w/ missing reply handle, convert msg to ActorMsg::Send {:?}",
                                &req
                            );
                            ActorMsg::Send(ActorSend {
                                id: req.id,
                                msg: req.msg,
                                msg_version: req.msg_version,
                            })
                        }
                    }
                    HollywoodMsg::Send(send) => ActorMsg::Send(ActorSend {
                        id: send.id,
                        msg: send.msg,
                        msg_version: send.msg_version,
                    }),
                    HollywoodMsg::Publish(publish) => ActorMsg::Subscribe(ActorSubscribe {
                        id: publish.id,
                        msg: publish.msg,
                        msg_version: publish.msg_version,
                    }),
                    _ => {
                        todo!("broker HollywoodMsg missing match arm");
                    }
                };

                // send to mailbox
                match mailbox_sender.send(actor_msg).await {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("broker failed to forward msg to actor mailbox: {:?}", &err);
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
        Ok(())
    }

    pub async fn run(&self, actor_name: String) -> Result<()> {
        for mailbox_name in &self.mailbox_names {
            let system_inbox = self.system_inbox.clone();
            let this_actor_name = actor_name.clone();
            let nats = self.nats.clone();
            let mailbox_name = mailbox_name.to_owned();
            let mailbox_sender = self.mailbox_sender.clone();
            let mailbox_max_size = self.mailbox_max_size.clone();
            let subscribe_type = self.subscribe_type.clone();
            tokio::spawn(async move {
                Broker::spawn(
                    this_actor_name,
                    system_inbox,
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
