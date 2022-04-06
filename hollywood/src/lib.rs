mod actor;
mod broker;
mod client;
mod common;

/// Types for defining and running Actors.
pub use actor::{
    run, Actor, ActorMailbox, Dispatch, DispatchResponse, DispatchType, Handle, Msg, RunOpts,
    SubscribeType,
};

pub mod prelude {
    pub mod actor {
        #[allow(unused_imports)]
        pub use super::super::{
            async_trait, run, Actor, Dispatch, DispatchResponse, DispatchType, Handle, Msg, Result,
            SubscribeType,
        };
    }
}

/// Hollywood Client. Use this if you need to
/// implement actor-to-actor communication.
pub use client::{mailbox, Client};

// Hollywood Config related things...
pub mod config;

// Hollywood env vars
pub mod env;

/// Defines common types so we can re-use
/// them in Actor implementations.
pub use anyhow::Result;
pub use async_trait::async_trait;
