// trait Serialize {
//     // Implement `into_bytes` to convert Self into bytes
//     fn into_bytes(&self) -> Result<Vec<u8>>;
// }

// trait Deserialize: std::marker::Sized {
//     // Implement `from_bytes` to convert bytes back to Self
//     fn from_bytes<'a>(bytes: &'a Vec<u8>) -> Result<Self>;
// }

// #[derive(buffalo::Read, buffalo::Write, Debug)]
// #[buffalo(size = "dynamic")]
// struct Example {
//     #[buffalo(id = 0, required)]
//     pub age: u16,
//     #[buffalo(id = 1, required)]
//     pub name: String,
// }

// impl Serialize for Example {
//     fn into_bytes(&self) -> Result<Vec<u8>> {
//         let mut writer = buffalo::Writer::new();
//         let age = self.age;
//         let name = writer.write(&self.name);
//         let output = writer.write(&ExampleWriter {
//             age: age,
//             name: name,
//         });
//         writer.write(&output);
//         let bytes = writer.into_bytes();
//         Ok(bytes)
//     }
// }

// impl Deserialize for Example {
//     fn from_bytes<'a>(bytes: &'a Vec<u8>) -> Result<Self> {
//         let reader = buffalo::read::<ExampleReader>(&bytes);
//         Ok(Self {
//             name: reader.name().to_string(),
//             age: reader.age(),
//         })
//     }
// }

// #[derive(buffalo::Read, buffalo::Write, Debug)]
// #[buffalo(size = "dynamic")]
// enum MsgVersion {
//     #[allow(unused)]
//     #[buffalo(id = 0)]
//     Prod(String),

//     #[allow(unused)]
//     #[buffalo(id = 1)]
//     Beta(String),
// }

// #[derive(buffalo::Read, buffalo::Write, Debug)]
// #[buffalo(size = "dynamic")]
// enum MsgType {
//     #[allow(unused)]
//     #[buffalo(id = 0)]
//     Hello,

//     #[allow(unused)]
//     #[buffalo(id = 1)]
//     World,

//     #[allow(unused)]
//     #[buffalo(id = 2)]
//     Say(String),
// }

// #[derive(buffalo::Read, buffalo::Write, Debug)]
// #[buffalo(size = "dynamic")]
// struct Msg {
//     #[allow(unused)]
//     #[buffalo(id = 0, required)]
//     id: String,

//     #[allow(unused)]
//     #[buffalo(id = 1, required)]
//     ty: MsgType,

//     #[allow(unused)]
//     #[buffalo(id = 2, required)]
//     version: MsgVersion,

//     #[allow(unused)]
//     #[buffalo(id = 3, required)]
//     inner: Vec<u8>,
// }

// impl Serialize for Msg {
//     fn into_bytes(&self) -> Result<Vec<u8>> {
//         let mut writer = buffalo::Writer::new();
//         let id = writer.write(&self.id);
//         let ty = match self.ty {
//             MsgType::Hello => &MsgTypeWriter::Hello,
//             MsgType::World => &MsgTypeWriter::World,
//             MsgType::Say(text) => &MsgTypeWriter::Say(writer.write(&text)),
//         };
//         let ty = writer.write(ty);

//         let version = writer.write(match self.version {
//             MsgVersion::Prod(v) => &MsgVersionWriter::Prod(writer.write(&v)),
//             MsgVersion::Beta(v) => &MsgVersionWriter::Beta(writer.write(&v)),
//         });

//         let inner = writer.write(&self.inner);
//         let output = writer.write(&MsgWriter {
//             id: id,
//             ty: ty,
//             version: version,
//             inner: inner,
//         });

//         writer.write(&output);
//         let bytes = writer.into_bytes();
//         Ok(bytes)
//     }
// }

// impl Deserialize for Msg {
//     fn from_bytes<'a>(bytes: &'a Vec<u8>) -> Result<Self> {
//         let reader = buffalo::read::<MsgReader>(&bytes);
//         Ok(Self {
//             id: reader.id().to_string(),
//             ty: match reader.ty() {
//                 MsgTypeReader::Hello(_) => MsgType::Hello,
//             },
//             version: match reader.version() {
//                 MsgVersionReader::Prod(version) => MsgVersion::Prod(version.read().to_string()),
//                 MsgVersionReader::Beta(version) => MsgVersion::Beta(version.read().to_string()),
//             },
//             inner: reader.inner().iter().collect::<Vec<_>>(),
//         })
//     }
// }

// fn main() -> Result<()> {
// let example = Example {
//     name: "greg".into(),
//     age: 21,
// };
// println!("example: {:?}", &example);

// let bytes = example.into_bytes()?;
// println!("example into_bytes: {:?}", &bytes);

// let example = Example::from_bytes(&bytes);
// println!("example: {:?}", &example);

// let msg = Msg {
//     id: "some-id".into(),
//     ty: MsgType::Hello,
//     version: MsgVersion::Prod("v1.0".into()),
//     inner: bytes,
// };
// println!("msg: {:?}", &msg);

// let msg_bytes = msg.into_bytes()?;
// println!("msg into_bytes: {:?}", &msg_bytes);

// let as_msg = Msg::from_bytes(&msg_bytes)?;
// println!("msg from_bytes: {:?}", &as_msg);

//     Ok(())
// }
