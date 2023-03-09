# Hollywood

Hollywood is an another Actor implementation written in Rust. Whereas most Rust Actor frameworks define a system of actors within parent/child processes, Hollywood doesn't do that. Instead, an actor runs as a standalone process and all actor communication goes through NAT's.

## Alpha-only

This is extremely-alpha-only at this point. So, if you want to try it, install it as a crate dependency from github.

In your `Cargo.toml`:

```
[dependencies]
hollywood = { git = "https://github.com/grippy/hollywood", package = "hollywood" }
hollywood-macro = { git = "https://github.com/grippy/hollywood", package = "hollywood-macro" }
```

## High-level concepts:

- Hollywood defines an `Actor` trait with the following functionality:

  - handle `send` type messages that don't return a response
  - handle `request` type messages where the caller expects a response
  - handle `subscribe` type messages sent over NAT's pubsub topics

- Hollywood defines a `Msg` trait that describes how to serialize/deserialize messages. Actor messages must implement `serde::Serialize` and `serde::Deserialize`. Hollywood defaults to using `serde::json` but can be overridden in the Msg trait implementation (see `into_bytes` and `from_bytes`).

- Hollywood Actors may define how to handle multiple message types. This could be useful for versioning messages. There's a macro that enables dispatching Actor messages by type.

- Hollywood implementations may define a config file for running a "system" of actors.
  The main use-case is running all actors locally with one command and then watching them for changes. But this could be expanded to aid build, testing, and deployments.

# Actors, Mailboxes, and Messages

Each actor should run as a standalone process. An actor has Broker which pulls messages from its mailbox. Currently, a mailbox is just a NATs subject (queue or pubsub).

## Actor Mailbox

Actor mailbox addresses (i.e. NATs subjects) follow this pattern:

- `hollywood://{system_name}@{actor_name}/{actor_version}::{msg_type}/{msg_version}`

Example for `MyActor` which handles `MyMsg v1.0`:

- `hollywood://prod@MyActor/v1.0::MyMsg/v1.0`

## Actor Messages

All actor messages are encoded as `HollywoodMsg` enums. From here, we define the type: `Send`, `Request` or `Publish` (if sending a pubsub message to a topic).

Within each type, contains the inner message consumed by an Actor. An actor should define how-to handle each HollywoodMsg variant for all inner message types and how to serialize/deserialize its own messages. By default, `HollywoodMsg` are passed using JSON (but this could change).

## Small footprint

Hollywood is ~1.3K lines of code. This could change once we added a proper System test harness but for now this is a pretty small footprint.

```
--------------------------------------------------------------------------------
 Language             Files        Lines        Blank      Comment         Code
--------------------------------------------------------------------------------
 Rust                    12         1614          167          141         1306
--------------------------------------------------------------------------------
```

## Examples

Here's how you run the examples. See the `examples` directory for code examples.

### Prerequisites

1. `docker` & `docker-compose`
2. `Rust`
3. `cargo watch`

### Start Docker

1. `docker-compose up`: this should run 3 nats containers and redis (which isn't a hollywood dependency but used as an example).

### Run the "examples" System

1. `cargo build --bin=hollywood-cli` to compile `hollywood-cli`
2. cd `examples`
3. `../target/debug/hollywood-cli dev --system=examples --config=hollywood.toml` (uses `cargo watch` to rebuild and re-run actors on code changes) or you can manually run each actor.

### Run the test-client

The `test-client` shows examples of how-to call actors.

1. cd `examples/test-client`
2. `RUST_LOG=info cargo run`

---
