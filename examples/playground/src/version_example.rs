// use anyhow::Result;

// trait Msg {}

// trait Version<V> {
//     fn version() -> V;
// }

// trait Actor<V> {
//     type Version;
//     fn versions(&self) -> Vec<Self::Version>;
//     fn route(&mut self, handle: HandleType, msg: Vec<u8>, version: Self::Version);
// }

// trait Handle<M, V>
// where
//     Self: Actor<V>,
//     M: Version<V>,
// {
//     type Msg;
//     type Version;
//     fn send(&mut self, msg: Self::Msg);
//     fn request(&mut self, msg: Self::Msg);
//     fn subscribe(&mut self, msg: Self::Msg);
// }

// #[allow(unused)]
// enum HandleType {
//     Send,
//     Request,
//     Subscribe,
// }

// enum MsgVersion {
//     V1,
//     V2,
//     #[allow(unused)]
//     V3,
// }

// struct M1 {}
// impl Version<MsgVersion> for M1 {
//     fn version() -> MsgVersion {
//         MsgVersion::V1
//     }
// }
// struct M2 {}
// impl Version<MsgVersion> for M2 {
//     fn version() -> MsgVersion {
//         MsgVersion::V2
//     }
// }

// struct MyActor {}
// impl Actor<MsgVersion> for MyActor {
//     type Version = MsgVersion;

//     // Actor defines which msg versions it handles
//     fn versions(&self) -> Vec<Self::Version> {
//         vec![MsgVersion::V1, MsgVersion::V2]
//     }

//     fn route(&mut self, ty: HandleType, _bytes: Vec<u8>, version: Self::Version) {
//         match version {
//             MsgVersion::V1 => {
//                 // deserialize msg to M1
//                 // let msg = self.deserialize::<M1>(&bytes);
//                 // <MyActor as Handle<M1, MsgVersion>>
//                 let msg = M1 {};
//                 match ty {
//                     HandleType::Send => <Self as Handle<M1, Self::Version>>::send(self, msg),
//                     HandleType::Request => <Self as Handle<M1, Self::Version>>::request(self, msg),
//                     HandleType::Subscribe => {
//                         <Self as Handle<M1, Self::Version>>::subscribe(self, msg)
//                     }
//                 }
//             }
//             MsgVersion::V2 => {
//                 let msg = M2 {};
//                 match ty {
//                     HandleType::Send => <Self as Handle<M2, Self::Version>>::send(self, msg),
//                     HandleType::Request => <Self as Handle<M2, Self::Version>>::request(self, msg),
//                     HandleType::Subscribe => {
//                         <Self as Handle<M2, Self::Version>>::subscribe(self, msg)
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
// }

// impl Handle<M1, MsgVersion> for MyActor {
//     type Msg = M1;
//     type Version = MsgVersion;
//     fn send(&mut self, _msg: Self::Msg) {}
//     fn request(&mut self, _msg: Self::Msg) {}
//     fn subscribe(&mut self, _msg: Self::Msg) {}
// }

// impl Handle<M2, MsgVersion> for MyActor {
//     type Msg = M2;
//     type Version = MsgVersion;
//     fn send(&mut self, _msg: Self::Msg) {}
//     fn request(&mut self, _msg: Self::Msg) {}
//     fn subscribe(&mut self, _msg: Self::Msg) {}
// }

// fn main() -> Result<()> {
//     Ok(())
// }
