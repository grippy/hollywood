use async_channel;

// SystemMsg is how we communicate across
// running threads
#[derive(Debug)]
pub enum SystemMsg {
    Health(async_channel::Sender<SystemMsg>),
    Heartbeat(SystemComponent, u128),
    Shutdown,
}

#[derive(Debug)]
pub enum SystemComponent {
    Actor,
    Agent,
    Broker,
}

pub type SystemSender = async_channel::Sender<SystemMsg>;
pub type SystemReceiver = async_channel::Receiver<SystemMsg>;
pub type SystemInbox = (SystemSender, SystemReceiver);
