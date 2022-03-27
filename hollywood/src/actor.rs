use crate::broker::Broker;
use crate::client;
use crate::common;
use crate::env::{hollywood_system, hollywood_system_nats_uri};
use anyhow::Result;
use async_channel;
use async_trait::async_trait;
use log::{error, info, warn};
use nats::asynk::Connection;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

#[allow(non_upper_case_globals)]
pub const VERSION_v1_0: &'static str = "v1.0";

/// Mailbox trait defines a common interface for
/// for sending and receiving messages.
#[async_trait]
trait Mailbox {
    type Msg;
    fn name(&self) -> String;
    fn sender(&self) -> async_channel::Sender<Self::Msg>;
    async fn recv(&mut self) -> Result<Self::Msg>;
}

/// Returns the mailbox name for a given system and actor.
/// This value is used as the Subject for reading/writing nats
/// messages.
pub(crate) fn mailbox_name(system_name: &String, actor_name: &String) -> String {
    format!("hollywood://{}@{}", system_name, actor_name)
}

/// Message type for sending nats requests
/// that expect a reply.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodRequest {
    pub id: String,
    pub msg: Vec<u8>,
    pub msg_version: String,
}

/// Message type for returning an Actor response.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodResponse {
    pub error: Option<String>,
    pub id: String,
    pub msg: Option<Vec<u8>>,
    pub msg_version: String,
}

/// Message type that sends a nats message
/// without expecting a reply.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodSend {
    pub id: String,
    pub msg: Vec<u8>,
    pub msg_version: String,
}

/// Message type that delivers a pubsub message
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodPublish {
    pub id: String,
    pub msg: Vec<u8>,
    pub msg_version: String,
}

/// Main message wrapper type for all messages
/// flowing through the system.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub(crate) enum HollywoodMsg {
    Request(HollywoodRequest),
    Response(HollywoodResponse),
    Send(HollywoodSend),
    Publish(HollywoodPublish),
}

impl Msg for HollywoodMsg {
    type Type = Self;
    const VERSION: &'static str = VERSION_v1_0;
}

// Do we need this message type?
// or at the least, can we make them...
// Request(HollywoodRequest, reply_id: String)
// Send(HollywoodSend)
// etc...
pub(crate) struct ActorRequest {
    pub id: String,
    pub msg_version: String,
    pub msg: Vec<u8>,
    pub reply_id: String,
}

pub(crate) struct ActorSend {
    pub id: String,
    pub msg_version: String,
    pub msg: Vec<u8>,
}

pub(crate) struct ActorSubscribe {
    pub id: String,
    pub msg_version: String,
    pub msg: Vec<u8>,
}

pub(crate) enum ActorMsg {
    Request(ActorRequest),
    Send(ActorSend),
    Subscribe(ActorSubscribe),
    #[allow(dead_code)]
    Health,
    #[allow(dead_code)]
    Shutdown,
}

pub(crate) type ActorSender = async_channel::Sender<ActorMsg>;
pub(crate) type ActorReceiver = async_channel::Receiver<ActorMsg>;

#[derive(Clone)]
pub enum SubscribeType {
    // This means actors are addressable
    // by the name of the Actor...
    // The actor should implement some variation
    // of send and request
    Queue,

    // This means actors are addressable
    // by pubsub subject. Use this type if multiple
    // actors should handle the same subject and message
    // type. Actors should implement the subscribe handler.
    Publish { subject: &'static str },
}

pub trait Msg
where
    Self: Sized + Serialize + DeserializeOwned,
{
    type Type;
    const VERSION: &'static str;
    fn name() -> &'static str {
        std::any::type_name::<&Self>()
            .split("::")
            .collect::<Vec<_>>()
            .last()
            .unwrap()
    }
    fn version() -> &'static str {
        Self::VERSION
    }

    fn dispatch_type() -> String {
        format!("{}/{}", Self::name(), Self::version())
    }

    fn into_bytes(&self) -> Result<Vec<u8>> {
        Ok(common::serialize(&self)?)
    }

    fn from_bytes(msg: &Vec<u8>) -> Result<Self> {
        Ok(common::deserialize::<Self>(msg)?)
    }
}

#[derive(Debug)]
pub enum DispatchType {
    Send,
    Request,
    Subscribe,
}

// DispatchResponse: (message version, message)
pub type DispatchResponse = (Option<&'static str>, Option<Vec<u8>>);

#[async_trait]
pub trait Dispatch {
    // This is used to configure which actor mailbox
    // version we should use...
    fn instance_dispatch_types(&self) -> Vec<String>;
    fn dispatch_types() -> Vec<String>;

    // `dispatch` a message to an actor
    async fn dispatch(
        &mut self,
        version: String,
        dispatch_type: &DispatchType,
        bytes: &Vec<u8>,
    ) -> Result<DispatchResponse>;
}

#[async_trait]
pub trait ActorMailbox
where
    Self: Actor,
{
    async fn mailbox<M: Msg>(
        system_name: String,
        nats_uri: String,
    ) -> Result<client::mailbox::Mailbox>;
    async fn mailbox_from_env<M: Msg>() -> Result<client::mailbox::Mailbox>;
}

/// Actor trait for defining an expected message
/// type and how to handle request/send type requests.
#[async_trait]
pub trait Actor {
    const VERSION: &'static str;

    fn version() -> &'static str {
        Self::VERSION
    }

    fn type_name() -> &'static str {
        std::any::type_name::<&Self>()
            .split("::")
            .collect::<Vec<_>>()
            .last()
            .unwrap()
    }

    fn type_name_version(&self) -> String {
        format!("{}/{}", Self::type_name(), Self::version())
    }

    // How we expect to run this actor..
    // If unimplemented, the default is `SubscribeType::Queue`
    fn instance_subscribe_type(&self) -> SubscribeType {
        Self::subscribe_type()
    }

    fn subscribe_type() -> SubscribeType {
        SubscribeType::Queue
    }
}

#[async_trait]
pub trait Handle<M>
where
    Self: Actor,
    M: Msg,
{
    type Msg;
    async fn send(&mut self, msg: Self::Msg) -> Result<()>;
    async fn request(&mut self, msg: Self::Msg) -> Result<Option<Self::Msg>>;
    async fn subscribe(&mut self, msg: Self::Msg) -> Result<()>;
}

// struct Broker {
//     actor_name: String,
//     mailbox_names: Vec<String>,
//     mailbox_sender: ActorSender,
//     mailbox_max_size: Option<u32>,
//     nats: Connection,
//     subscribe_type: SubscribeType,
// }

// impl Broker {
//     fn new(
//         actor_name: String,
//         mailbox_names: Vec<String>,
//         mailbox_sender: ActorSender,
//         mailbox_max_size: Option<u32>,
//         nats: Connection,
//         subscribe_type: SubscribeType,
//     ) -> Self {
//         Self {
//             actor_name: actor_name,
//             mailbox_names: mailbox_names,
//             mailbox_sender: mailbox_sender,
//             mailbox_max_size: mailbox_max_size,
//             nats: nats,
//             subscribe_type: subscribe_type,
//         }
//     }

//     async fn spawn(
//         actor_name: String,
//         mailbox_max_size: Option<u32>,
//         mailbox_name: String,
//         mailbox_sender: ActorSender,
//         nats: Connection,
//         subscribe_type: SubscribeType,
//     ) -> Result<()> {
//         let source = match subscribe_type {
//             SubscribeType::Queue => {
//                 info!(
//                     "{} agent subscribing to queue subject {:?}",
//                     &actor_name, &mailbox_name
//                 );
//                 nats
//                     // use the mailbox name as the group
//                     .queue_subscribe(&mailbox_name, &mailbox_name)
//                     .await?
//             }
//             SubscribeType::Publish { subject } => {
//                 info!(
//                     "{} agent subscribing to pubsub subject {:?}",
//                     &actor_name, &subject
//                 );
//                 nats.subscribe(&subject).await?
//             }
//         };

//         let max_size = if mailbox_max_size.is_some() {
//             mailbox_max_size.unwrap()
//         } else {
//             0
//         };
//         let mut backoff = 0;
//         loop {
//             // slow down how fast we read nats messages
//             // if mailbox is full
//             loop {
//                 if max_size > 0 && mailbox_sender.len() > max_size as usize {
//                     sleep(Duration::from_millis(100)).await;
//                 } else {
//                     break;
//                 }
//             }

//             if let Some(nats_msg) = source.try_next() {
//                 // deserialize nats_msg.data here
//                 let hollywood_msg: HollywoodMsg = match serde_json::from_slice(&nats_msg.data) {
//                     Ok(msg) => msg,
//                     Err(err) => {
//                         error!("deserializing nats msg to HollywoodMsg: {:?}", &err);
//                         continue;
//                     }
//                 };

//                 // We should only have request/send here
//                 // HollywoodMsg::Response type is only
//                 let (msg_id, msg_version, msg) = match hollywood_msg {
//                     HollywoodMsg::Request(resp) => (resp.id, resp.msg_version, resp.msg),
//                     HollywoodMsg::Send(send) => (send.id, send.msg_version, send.msg),
//                     HollywoodMsg::Publish(publish) => {
//                         (publish.id, publish.msg_version, publish.msg)
//                     }
//                     _ => {
//                         todo!("HollywoodMsg: not yet implemented");
//                     }
//                 };

//                 // create ActorMsg.. if nats msg
//                 // has a reply handle then send a nats request
//                 // so we can route the response back to the caller
//                 let msg = if nats_msg.reply.is_some() {
//                     ActorMsg::Request(ActorRequest {
//                         id: msg_id,
//                         msg: msg,
//                         msg_version: msg_version,
//                         reply_id: nats_msg.reply.unwrap(),
//                     })
//                 } else {
//                     // send-type: queue or pubsub?
//                     match subscribe_type {
//                         SubscribeType::Queue => ActorMsg::Send(ActorSend {
//                             id: msg_id,
//                             msg: msg,
//                             msg_version: msg_version,
//                         }),
//                         _ => ActorMsg::Subscribe(ActorSubscribe {
//                             id: msg_id,
//                             msg: msg,
//                             msg_version: msg_version,
//                         }),
//                     }
//                 };

//                 // send to mailbox
//                 match mailbox_sender.send(msg).await {
//                     Ok(_) => {}
//                     Err(err) => {
//                         warn!("failed to forward msg to actor mailbox: {:?}", &err);
//                     }
//                 }
//                 backoff = 0;
//             } else {
//                 // backoff max is 1sec
//                 backoff = std::cmp::min(backoff + 10, 1000);
//             }

//             if backoff > 0 {
//                 sleep(Duration::from_millis(backoff)).await;
//             }
//         }
//         // Ok(())
//     }

//     async fn run(&self) -> Result<()> {
//         // spawn broker for each mailbox
//         for mailbox_name in &self.mailbox_names {
//             let actor_name = self.actor_name.clone();
//             let nats = self.nats.clone();
//             let mailbox_name = mailbox_name.to_owned();
//             let mailbox_sender = self.mailbox_sender.clone();
//             let mailbox_max_size = self.mailbox_max_size.clone();
//             let subscribe_type = self.subscribe_type.clone();
//             tokio::spawn(async move {
//                 Broker::spawn(
//                     actor_name,
//                     mailbox_max_size,
//                     mailbox_name,
//                     mailbox_sender,
//                     nats,
//                     subscribe_type,
//                 )
//                 .await
//             });
//         }
//         Ok(())
//     }
// }

/// Agent
struct Agent<A: Actor + Dispatch> {
    system_name: String,
    actor: A,
    max_size: Option<u32>,
    sender: ActorSender,
    receiver: ActorReceiver,
    nats: Connection,
}

impl<A: Actor + Dispatch> Agent<A> {
    fn new(system_name: String, actor: A, max_size: Option<u32>, nats: Connection) -> Self {
        let (tx, rx) = async_channel::unbounded();
        Agent {
            system_name: system_name,
            actor: actor,
            max_size: max_size,
            sender: tx,
            receiver: rx,
            nats: nats,
        }
    }

    fn sender(&self) -> ActorSender {
        self.sender.clone()
    }

    async fn recv(&mut self) -> Result<ActorMsg> {
        match self.receiver.recv().await {
            Ok(msg) => Ok(msg),
            Err(err) => Err(err.into()),
        }
    }

    /// Publish the response to an actor request
    async fn reply(&mut self, reply_id: String, msg: HollywoodMsg) {
        match serde_json::to_vec(&msg) {
            Ok(msg) => match self.nats.publish(&reply_id[..], msg).await {
                Err(err) => {
                    error!("sending response to nats: {:?}", &err);
                }
                _ => {}
            },
            Err(err) => {
                error!("serializing request response: {:?}", &err);
            }
        }
    }

    async fn handle_mailbox_msg(&mut self, mailbox_msg: ActorMsg) {
        match mailbox_msg {
            ActorMsg::Health => {
                // TODO: figure out what a health check means
                todo!("implement health check");
            }
            ActorMsg::Shutdown => {
                todo!("implement shutdown");
            }
            ActorMsg::Request(req) => {
                self.handle_msg(
                    req.id,
                    req.msg_version,
                    &DispatchType::Request,
                    &req.msg,
                    Some(req.reply_id),
                )
                .await;
            }
            ActorMsg::Send(send) => {
                self.handle_msg(
                    send.id,
                    send.msg_version,
                    &DispatchType::Send,
                    &send.msg,
                    None,
                )
                .await;
            }
            ActorMsg::Subscribe(sub) => {
                self.handle_msg(
                    sub.id,
                    sub.msg_version,
                    &DispatchType::Subscribe,
                    &sub.msg,
                    None,
                )
                .await;
            }
        }
    }

    /// Wrapper for handling send/request/subscribe type messages
    async fn handle_msg(
        &mut self,
        id: String,
        version: String,
        dispatch_type: &DispatchType,
        msg: &Vec<u8>,
        reply_id: Option<String>,
    ) {
        let result = Dispatch::dispatch(&mut self.actor, version, &dispatch_type, msg).await;
        let hollywood_msg = match result {
            Ok((msg_version, msg)) => match dispatch_type {
                DispatchType::Request => {
                    let resp = HollywoodResponse {
                        id: id,
                        msg_version: msg_version.unwrap_or("unknown_version").to_string(),
                        msg: msg,
                        error: None,
                    };
                    HollywoodMsg::Response(resp)
                }
                _ => return,
            },
            Err(err) => match dispatch_type {
                DispatchType::Request => {
                    let resp = HollywoodResponse {
                        id: id,
                        msg_version: "".to_string(),
                        msg: None,
                        error: Some(err.to_string()),
                    };
                    HollywoodMsg::Response(resp)
                }
                _ => return,
            },
        };

        if reply_id.is_some() {
            let reply_id = reply_id.unwrap();
            self.reply(reply_id, hollywood_msg).await;
        }
    }

    /// Run implements a basic runtime for running an
    /// agent (broker and actor mailbox consumer) for given
    /// system and actor instance.
    async fn run(&mut self) -> Result<()> {
        // run broker here...
        let system_name = &self.system_name;
        let actor_type_name_version = &self.actor.type_name_version();

        // prepare the broker for each mailbox
        let mailbox_sender = self.sender();
        let mailbox_max_size = self.max_size.clone();
        let nats = self.nats.clone();
        let subscribe_type = self.actor.instance_subscribe_type().clone();
        let mailbox_names = self
            .actor
            .instance_dispatch_types()
            .into_iter()
            .map(|msg_version| {
                let actor_name = format!("{}::{}", &actor_type_name_version, &msg_version);
                mailbox_name(&system_name, &actor_name)
            })
            .collect::<Vec<_>>();

        // run broker...
        let broker = Broker::new(
            actor_type_name_version.to_owned(),
            mailbox_names,
            mailbox_sender,
            mailbox_max_size,
            nats,
            subscribe_type,
        );

        let _ = broker.run().await?;

        // run the mailbox receiver...
        // TODO: can we make a pool of these here?
        loop {
            if let Ok(mailbox_msg) = self.recv().await {
                self.handle_mailbox_msg(mailbox_msg).await
            }
        }
        // Ok(())
    }
}

/// RunOpts defines an actor instance and
/// a set of common configuration options.
pub struct RunOpts<A: Actor + Dispatch> {
    /// The system name we want to connect too.
    system_name: String,
    /// The actor instance we want to run.
    actor: A,
    /// The maximum size of unprocessed messages
    /// for the actor mailbox. Default is None which means
    /// the mailbox is unbounded.
    actor_mailbox_max_size: Option<u32>,
    /// The nats connection string as a uri.
    nats_uri: String,
}

impl<A: Actor + Dispatch> RunOpts<A> {
    pub fn new(system_name: String, actor: A, nats_uri: String) -> Self {
        Self {
            system_name: system_name,
            actor: actor,
            actor_mailbox_max_size: None,
            nats_uri,
        }
    }

    // Create RunOpts from hollywood env variables
    pub fn from_env(actor: A) -> Result<Self> {
        let system_name = hollywood_system()?;
        let nats_uri = hollywood_system_nats_uri(system_name.clone())?;
        info!(
            "from_env system:{:?}, nats_uri:{:?}",
            &system_name, &nats_uri
        );
        Ok(Self {
            system_name: system_name,
            actor: actor,
            actor_mailbox_max_size: None,
            nats_uri,
        })
    }

    pub fn with_actor_mailbox_max_size(mut self, size: Option<u32>) -> Self {
        self.actor_mailbox_max_size = size;
        self
    }
}

/// This is the public interface for running an actor.
pub async fn run<A: Actor + Dispatch>(opts: RunOpts<A>) -> Result<()> {
    let system_name = opts.system_name;
    let actor = opts.actor;
    let actor_mailbox_max_size = opts.actor_mailbox_max_size;
    let nats_uri = &opts.nats_uri;

    // TODO: set a max retries here...
    let mut result = nats::asynk::connect(&nats_uri).await;
    loop {
        match result {
            Err(err) => {
                warn!(
                    "error connecting {} agent to nats: #{:?}",
                    A::type_name(),
                    &err
                );
                sleep(Duration::from_millis(1000)).await;
                result = nats::asynk::connect(&nats_uri).await;
            }
            Ok(_) => {
                info!("{} agent connected to nats...", A::type_name());
                break;
            }
        }
    }
    // unwrap the nats client
    let nats_client = result.unwrap();
    info!("{} agent running", A::type_name());
    let mut agent = Agent::new(system_name, actor, actor_mailbox_max_size, nats_client);
    agent.run().await
}
