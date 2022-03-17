mod actor;
mod client;
mod common;

/// Types for defining and running Actors.
pub use actor::{run, Actor, RunOpts, SubscribeType};

/// Hollywood Client. Use this if you need to
/// implement actor-to-actor communication.
pub use client::Client;

// Hollywood Config related things...
pub mod config;

// Hollywood env vars
pub mod env;

/// Defines common types so we can re-use
/// them in Actor implementations.
pub use anyhow::Result;
pub use async_trait::async_trait;
