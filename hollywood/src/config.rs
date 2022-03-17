use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::io::{Error, ErrorKind};

// This file describes how to parse
// hollywood.toml files.

#[derive(Deserialize, Debug)]
pub struct Config {
    // A list of system names and some configuration
    // values for each one.
    pub system: Vec<System>,
    // A list of actors paths
    pub actor: Vec<Actor>,
}

impl Config {
    pub fn load(file_path: &dyn AsRef<std::path::Path>) -> Result<Self, Error> {
        let file = match fs::read(file_path) {
            Ok(v) => String::from_utf8(v),
            Err(err) => return Err(Error::new(ErrorKind::InvalidInput, err)),
        };
        let toml = match file {
            Ok(toml) => toml,
            Err(err) => return Err(Error::new(ErrorKind::InvalidInput, err)),
        };
        match toml::from_str(&toml[..]) {
            Ok(cfg) => Ok(cfg),
            Err(err) => Err(Error::new(ErrorKind::InvalidInput, err)),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct System {
    // the name of the system
    pub name: String,
    // the nats uri to connect too
    pub nats_uri: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Actor {
    // The name of the actor
    pub name: String,
    // The dev configuration
    pub dev: Option<ActorDev>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActorDev {
    // the path to the actor crate
    pub path: String,
    // the name of the actor binary
    pub bin: String,
    // a list of env variables to call cargo watch with
    pub env: Vec<String>,
    // a list of path directories relative the config file path
    pub watch: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let toml = r#"
[[system]]
name = "system1"
nats_uri = "nats://system1"

[[system]]
name = "system2"
nats_uri = "nats://system2"

[[actor]]
name = "Actor1"

[actor.dev]
bin = "actor-1"
path = "examples/actor-1"
env = ["ENV1=env1"]
watch = ["dep1", "dep2"]

[[actor]]
name = "Actor2"

[actor.dev]
bin = "actor-2"
path = "examples/actor-2"
env = ["ENV2=env2"]
watch = ["dep2", "dep3"]"#;
        let config: Config = toml::from_str(toml).unwrap();
        println!("config: {:#?}", &config);
    }
}
