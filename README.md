# Hollywood

Hollywood is an another Actor implementation written in Rust. Whereas most Rust Actor frameworks define a system of actors within one parent process, Hollywood doesn't do that. Instead, an actor runs as a standalone process and all actor communication goes through NAT's.

## High-level concepts:

- Hollywood defines an `Actor` trait with the following functionality:
    - handle `send` type messages that don't return a response
    - handle `request` type messages where the caller expects a response
    - handle `subscribe` type messages for message sent to pubsub topics
- Hollywood defines a `Msg` trait that describes how to serialize/deserialize messages.
- Hollywood Actors may define how to handle multiple message types. This could be useful for versioning messages. There's a macro that enables dispatching messages to correct Handle implementation.
- Hollywood implementations may define a config file for running a "system" of actors.
  The main use-case for this is configuring the dev environment for running and testing actor changes locally. But this could be expanded to aid build, testing, and deployments.

## Small footprint

Hollywood is ~1.3K lines of code.
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

### Run the "System"

1. `cargo build --bin=hollywood-cli` to compile `hollywood-cli`
2. cd `examples`
3. `../target/debug/hollywood-cli dev --system=examples --config=hollywood.toml` (uses `cargo watch` to rebuild and run actors on code changes) or you can manually run each actor.

### Run the test-client

The `test-client` shows examples of how-to make calls to actors.

1. cd `examples/test-client`
2. `RUST_LOG=info cargo run`

---
