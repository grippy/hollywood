use crate::env::{hollywood_system, hollywood_system_nats_uri};
use anyhow::Result;
use async_channel;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use nats::asynk::Connection;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

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
}

/// Message type for returning an Actor response.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodResponse {
    pub id: String,
    pub msg: Option<Vec<u8>>,
    pub error: Option<String>,
}

/// Message type that sends a nats message
/// without expecting a reply.
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodSend {
    pub id: String,
    pub msg: Vec<u8>,
}

/// Message type that delivers a pubsub message
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct HollywoodPublish {
    pub id: String,
    pub msg: Vec<u8>,
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

// Do we need this message type?
// or at the least, can we make them...
// Request(HollywoodRequest, reply_id: String)
// Send(HollywoodSend)
// etc...
enum ActorMailboxMsg {
    Request {
        id: String,
        msg: Vec<u8>,
        reply_id: String,
    },
    Send {
        id: String,
        msg: Vec<u8>,
    },
    Subscribe {
        id: String,
        msg: Vec<u8>,
    },
    #[allow(dead_code)]
    Health,
    #[allow(dead_code)]
    Shutdown,
}

/// ActorMailbox stores messages received by the Agent broker
struct ActorMailbox {
    name: String,
    max_size: Option<u32>,
    sender: async_channel::Sender<ActorMailboxMsg>,
    receiver: async_channel::Receiver<ActorMailboxMsg>,
    nats: Connection,
}

impl ActorMailbox {
    fn new(name: String, max_size: Option<u32>, nats: Connection) -> Self {
        let (tx, rx) = async_channel::unbounded();
        ActorMailbox {
            name: name,
            max_size,
            sender: tx,
            receiver: rx,
            nats: nats,
        }
    }
}

/// Actor mailbox implementation
///
/// The agent broker messages from the actor nats
/// queue into this data structure. From here,
/// messages are received and handled by
/// an actor.
#[async_trait]
impl Mailbox for ActorMailbox {
    type Msg = ActorMailboxMsg;

    fn name(&self) -> String {
        self.name.clone()
    }

    fn sender(&self) -> async_channel::Sender<Self::Msg> {
        self.sender.clone()
    }

    async fn recv(&mut self) -> Result<Self::Msg> {
        match self.receiver.recv().await {
            Ok(msg) => Ok(msg),
            Err(err) => Err(err.into()),
        }
    }
}

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

/// Actor trait for defining an expected message
/// type and how to handle request/send type requests.
#[async_trait]
pub trait Actor {
    /// The mailbox message type..
    /// This should be serializable to/from Vec<u8>
    type Msg: Serialize + DeserializeOwned;

    // The name of an actor
    fn name(&self) -> &'static str;

    // How we expect to run this actor..
    // If unimplemented, the default is `SubscribeType::Queue`
    fn subscribe_type(&self) -> SubscribeType {
        SubscribeType::Queue
    }
    // Implement this to handle serializing outbound

    // response message. The default serialization type
    // is serde_json
    fn serialize(&self, msg: &Self::Msg) -> Result<Vec<u8>>
    where
        Self::Msg: Serialize,
    {
        match serde_json::to_vec(msg) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(err.into()),
        }
    }
    /// Implement this to handle deserializing inbound
    /// request/send messages
    fn deserialize<'a>(&self, msg: &'a Vec<u8>) -> Result<Self::Msg>
    where
        Self::Msg: Deserialize<'a>,
    {
        match serde_json::from_slice(msg) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(err.into()),
        }
    }

    // // TBD: we might not need this for now
    // async fn initialize(&mut self) -> Result<()>;
    /// Implement this for handling `request` type messages.
    async fn request(&mut self, msg: Self::Msg) -> Result<Option<Self::Msg>>;
    /// Implement this for handling `send` type messages.
    async fn send(&mut self, msg: Self::Msg) -> Result<()>;
    /// Implement this for handling `subscribe` type messages.
    async fn subscribe(&mut self, msg: Self::Msg) -> Result<()>;
}

/// Agent is responsible for processing
/// actor mailbox messages and passing them
/// to an actor.
struct Agent<A: Actor> {
    mailbox: ActorMailbox,
    actor: A,
}

impl<A: Actor> Agent<A> {
    fn new(mailbox: ActorMailbox, actor: A) -> Self {
        Agent {
            actor: actor,
            mailbox: mailbox,
        }
    }

    /// Publish the response to an actor request
    async fn reply(&mut self, reply_id: String, msg: HollywoodMsg) {
        match serde_json::to_vec(&msg) {
            Ok(msg) => match self.mailbox.nats.publish(&reply_id[..], msg).await {
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

    async fn handle_mailbox_msg(&mut self, mailbox_msg: ActorMailboxMsg) {
        match mailbox_msg {
            ActorMailboxMsg::Health => {
                // TODO: figure out what a health check means
                todo!("implement health check");
            }
            ActorMailboxMsg::Shutdown => {
                todo!("implement shutdown");
            }
            ActorMailboxMsg::Request { id, msg, reply_id } => {
                self.handle_msg(id, &msg, Some(reply_id), false, false)
                    .await;
            }
            ActorMailboxMsg::Send { id, msg } => {
                self.handle_msg(id, &msg, None, true, false).await;
            }
            ActorMailboxMsg::Subscribe { id, msg } => {
                self.handle_msg(id, &msg, None, false, true).await;
            }
        }
    }

    /// Wrapper for handling both send & request type messages
    async fn handle_msg(
        &mut self,
        id: String,
        msg: &Vec<u8>,
        reply_id: Option<String>,
        send: bool,
        subscribe: bool,
    ) {
        match self.actor.deserialize(msg) {
            Ok(msg) => {
                // existence of a reply_id means we have
                // an actor request and expect a reply
                // that should be sent back through nats
                if reply_id.is_some() {
                    let reply_id = reply_id.unwrap();
                    match self.actor.request(msg).await {
                        Ok(resp) => {
                            debug!("agent request msg id {} success", &id);
                            match self.actor.serialize(&resp.unwrap()) {
                                Ok(msg) => {
                                    let resp = HollywoodResponse {
                                        id: id,
                                        msg: Some(msg),
                                        error: None,
                                    };
                                    let msg = HollywoodMsg::Response(resp);
                                    self.reply(reply_id, msg).await;
                                }
                                Err(err) => {
                                    error!("agent serialize msg id {} err {:?}", &id, &err);
                                    let resp = HollywoodResponse {
                                        id: id,
                                        msg: None,
                                        error: Some(err.to_string()),
                                    };
                                    let msg = HollywoodMsg::Response(resp);
                                    self.reply(reply_id, msg).await;
                                }
                            }
                        }
                        Err(err) => {
                            error!("agent request msg id {} err: {:?}", &id, &err);
                            let resp = HollywoodResponse {
                                id: id,
                                msg: None,
                                error: Some(err.to_string()),
                            };
                            let msg = HollywoodMsg::Response(resp);
                            self.reply(reply_id, msg).await;
                        }
                    }
                } else {
                    if send {
                        // call actor.send
                        match self.actor.send(msg).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("agent send msg id, {} err: {:?}", &id, &err);
                            }
                        }
                    } else if subscribe {
                        // call actor.subscribe
                        match self.actor.subscribe(msg).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("agent subscribe msg id, {} err: {:?}", &id, &err);
                            }
                        }
                    }
                }
            }
            Err(err) => {
                error!("agent deserialize msg id {} err: {:?}", &id, &err);
                if reply_id.is_some() {
                    let resp = HollywoodResponse {
                        id: id,
                        msg: None,
                        error: Some(err.to_string()),
                    };
                    let msg = HollywoodMsg::Response(resp);
                    self.reply(reply_id.unwrap(), msg).await;
                }
            }
        }
    }

    /// Run implements a basic runtime for running an
    /// agent (broker and actor mailbox consumer) for given
    /// system and actor instance.
    async fn run(&mut self) -> Result<()> {
        // run broker here...
        let nats = self.mailbox.nats.clone();
        let mailbox_name = self.mailbox.name();
        let mailbox_sender = self.mailbox.sender();
        let mailbox_max_size = self.mailbox.max_size.clone();
        let subscribe_type = self.actor.subscribe_type().clone();

        let source = match subscribe_type {
            SubscribeType::Queue => {
                info!(
                    "{} agent subscribing to queue subject {:?}",
                    &self.actor.name(),
                    &mailbox_name
                );
                nats
                    // use the mailbox name as the group
                    .queue_subscribe(&mailbox_name, &mailbox_name)
                    .await?
            }
            SubscribeType::Publish { subject } => {
                info!(
                    "{} agent subscribing to pubsub subject {:?}",
                    &self.actor.name(),
                    &subject
                );
                nats.subscribe(&subject).await?
            }
        };

        // spawn broker
        tokio::spawn(async move {
            let max_size = if mailbox_max_size.is_some() {
                mailbox_max_size.unwrap()
            } else {
                0
            };
            let mut backoff = 0;
            loop {
                // should we slow down how fast we read nats messages
                // if mailbox is full?
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
                    let (msg_id, msg) = match hollywood_msg {
                        HollywoodMsg::Request(resp) => (resp.id, resp.msg),
                        HollywoodMsg::Send(send) => (send.id, send.msg),
                        HollywoodMsg::Publish(publish) => (publish.id, publish.msg),
                        _ => {
                            todo!("HollywoodMsg: not yet implemented");
                        }
                    };

                    // create ActorMailboxMsg.. if nats msg
                    // has a reply handle then we make this a request
                    // so we can route the response back to the caller
                    let msg = if nats_msg.reply.is_some() {
                        ActorMailboxMsg::Request {
                            id: msg_id,
                            msg: msg,
                            reply_id: nats_msg.reply.unwrap(),
                        }
                    } else {
                        // send-type: queue or pubsub?
                        match subscribe_type {
                            SubscribeType::Queue => ActorMailboxMsg::Send {
                                id: msg_id,
                                msg: msg,
                            },
                            _ => ActorMailboxMsg::Subscribe {
                                id: msg_id,
                                msg: msg,
                            },
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
        });

        // run the mailbox receiver...
        // TODO: can we make a pool of these here?
        loop {
            if let Ok(mailbox_msg) = self.mailbox.recv().await {
                self.handle_mailbox_msg(mailbox_msg).await
            }
        }
        // Ok(())
    }
}

/// RunOpts defines an actor instance and
/// a set of common configuration options.
pub struct RunOpts<A: Actor> {
    /// The system name we want to connect too.
    system_name: String,
    /// The actor name we want to run as. This derived
    /// from the Actor trait `name()` method.
    actor_name: String,
    /// The actor instance we want to run.
    actor: A,
    /// The maximum size of unprocessed messages
    /// for the actor mailbox. Default is None which means
    /// the mailbox is unbounded.
    actor_mailbox_max_size: Option<u32>,
    /// The nats connection string as a uri.
    nats_uri: String,
}

impl<A: Actor> RunOpts<A> {
    pub fn new(system_name: String, actor: A, nats_uri: String) -> Self {
        Self {
            system_name: system_name,
            actor_name: actor.name().to_string(),
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
            actor_name: actor.name().to_string(),
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
pub async fn run<A: Actor>(opts: RunOpts<A>) -> Result<()> {
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
                    &actor.name(),
                    &err
                );
                sleep(Duration::from_millis(1000)).await;
                result = nats::asynk::connect(&nats_uri).await;
            }
            Ok(_) => {
                info!("{} agent connected to nats...", &actor.name());
                break;
            }
        }
    }
    // unwrap the nats client
    let nats_client = result.unwrap();

    // configure actor mailbox
    let name = mailbox_name(&opts.system_name, &opts.actor_name);
    let mailbox = ActorMailbox::new(name, actor_mailbox_max_size, nats_client);

    info!("{} agent running", &actor.name());
    let mut agent = Agent::new(mailbox, actor);
    agent.run().await
}
