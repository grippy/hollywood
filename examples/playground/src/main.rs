use anyhow::Result;
use hollywood_macro::{dispatch, dispatch_types};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[allow(non_upper_case_globals)]
const VERSION_v1_0: &'static str = "v1.0";
#[allow(non_upper_case_globals)]
const VERSION_v2_0: &'static str = "v2.0";

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

trait Msg
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
        Ok(serialize(&self)?)
    }

    fn from_bytes(msg: &Vec<u8>) -> Result<Self> {
        Ok(deserialize::<Self>(msg)?)
    }
}

#[allow(unused)]
enum Dispatch {
    Send,
    Request,
    Subscribe,
}

trait Actor {
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

    fn name_version() -> String {
        format!("{}/{}", Self::name(), Self::version())
    }

    // This is used to configure which actor mailbox
    // version we should use...
    fn dispatch_types() -> Vec<String>;

    fn dispatch(
        &mut self,
        _version: &'static str,
        _dispatch: Dispatch,
        _bytes: &Vec<u8>,
    ) -> Result<Option<Vec<u8>>>;
}

trait Handle<M>
where
    Self: Actor,
    M: Msg,
{
    type Msg;
    fn send(&mut self, _msg: Self::Msg) -> Result<()> {
        Ok(())
    }
    fn request(&mut self, _msg: Self::Msg) -> Result<Option<Self::Msg>> {
        Ok(None)
    }
    fn subscribe(&mut self, _msg: Self::Msg) -> Result<()> {
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
struct Msg1 {
    name: String,
}

impl Msg for Msg1 {
    type Type = Self;
    const VERSION: &'static str = VERSION_v1_0;
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
struct Msg2 {
    name: String,
    phone: String,
}
impl Msg for Msg2 {
    type Type = Self;
    const VERSION: &'static str = VERSION_v2_0;
}

#[derive(Default)]
struct MyActor {}

impl Actor for MyActor {
    const VERSION: &'static str = VERSION_v1_0;

    #[dispatch_types(Msg1, Msg2)]
    fn dispatch_types() {}

    #[dispatch(Msg1, Msg2)]
    fn dispatch() {}
}

impl Handle<Msg1> for MyActor {
    type Msg = Msg1;

    fn send(&mut self, msg: Self::Msg) -> Result<()> {
        println!("MyActor::send Msg1: {:?}", &msg);
        Ok(())
    }
    fn request(&mut self, _msg: Self::Msg) -> Result<Option<Self::Msg>> {
        println!("MyActor::request Msg1");
        Ok(None)
    }
    fn subscribe(&mut self, _msg: Self::Msg) -> Result<()> {
        println!("MyActor::subscribe Msg1");
        Ok(())
    }
}

impl Handle<Msg2> for MyActor {
    type Msg = Msg2;

    fn send(&mut self, msg: Self::Msg) -> Result<()> {
        println!("MyActor::send Msg2: {:?}", &msg);
        Ok(())
    }
    fn request(&mut self, _msg: Self::Msg) -> Result<Option<Self::Msg>> {
        println!("MyActor::request Msg2");
        Ok(None)
    }
    fn subscribe(&mut self, _msg: Self::Msg) -> Result<()> {
        println!("MyActor::subscribe Msg2");
        Ok(())
    }
}

fn main() -> Result<()> {
    println!(
        "{}::dispatch_types: {:?}",
        MyActor::name_version(),
        MyActor::dispatch_types()
    );

    println!(
        "{}::{}",
        MyActor::name_version(),
        <MyActor as Handle<Msg1>>::Msg::dispatch_type(),
    );
    println!(
        "{}::{}",
        MyActor::name_version(),
        <MyActor as Handle<Msg2>>::Msg::dispatch_type(),
    );

    let msg1 = Msg1 {
        name: "hello".into(),
    };
    println!("{:?}", &msg1);

    let msg1_bytes = &msg1.into_bytes()?;
    println!("{:?}", &msg1_bytes);
    println!("{:?}", Msg1::from_bytes(&msg1_bytes)?);

    let msg2 = Msg2 {
        name: "hello".into(),
        phone: "555-1212".into(),
    };
    println!("{:?}", &msg2);

    let msg2_bytes = &msg2.into_bytes()?;
    println!("{:?}", &msg2_bytes);
    println!("{:?}", Msg2::from_bytes(&msg2_bytes)?);

    let mut actor = MyActor::default();
    actor.dispatch(Msg1::VERSION, Dispatch::Send, &msg1_bytes)?;
    actor.dispatch(Msg1::VERSION, Dispatch::Request, &msg1_bytes)?;
    actor.dispatch(Msg1::VERSION, Dispatch::Subscribe, &msg1_bytes)?;

    actor.dispatch(Msg2::VERSION, Dispatch::Send, &msg2_bytes)?;
    actor.dispatch(Msg2::VERSION, Dispatch::Request, &msg2_bytes)?;
    actor.dispatch(Msg2::VERSION, Dispatch::Subscribe, &msg2_bytes)?;

    actor.dispatch(Msg1::VERSION, Dispatch::Send, &msg2_bytes)?;
    actor.dispatch(Msg2::VERSION, Dispatch::Send, &msg1_bytes)?;

    Ok(())
}
