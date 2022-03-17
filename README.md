# Hollywood

Hollywood is an Actor runtime that uses NAT's as the datastore for actor mailboxes.
An actor is compiled as a binary that runs an async tokio event loop to read messages from an actor mailbox.

## High-level concepts:

- Hollywood defines an `Actor` trait with the following functionality:
    - `serialize`/`deserialize` inbound and outbound messages
    - handle `send` type messages that don't return a response
    - handle `request` type messages where the caller expects a response
    - handle `subscribe` type messages for message sent to pubsub subjects

- Hollywood config file for defining a "system" of actors. The main use-case for this is configuring the dev environment for running and testing actor changes locally.

## Small footprint

Hollywood is < 1K lines of code.

## Examples

Here's how you run the examples. See the `examples` directory for code examples.

### Prerequisites

1. `docker`/`docker-compose`
2. `Rust`
3. `cargo watch`

### Start Docker

1. `docker-compose up`: this should run 3 nats containers and redis (which isn't a hollywood dependency)

### Run the "System"

1. `cargo build --bin=hollywood-cli` to compile `hollywood-cli`
2. cd `examples`
3. `../target/debug/hollywood-cli dev --system=examples --config=hollywood.toml` (uses `cargo watch` to rebuild and run actors when they code changes) or you can manually run each actor.

### Run the test-client

The `test-client` shows examples of how-to make calls to actors.

1. cd `examples/test-client`
2. `RUST_LOG=info cargo run`


---


## TODO

- [ ] Test coverage is non-existent
- [ ] Health
- [ ] Shutdown
- [x] Add Msg Id
- [x] Run options:
    - [x] Max queue size

- [x] Uber runtime across multiple actors
    - [x] Process launcher
    - [x] Config to define system
    - [x] Configure HOLLYWOOD_* variables
    - [x] RunOptions::from_env
    - [x] Client::from_env

- [x] Client request should define one type which
      is the same for input and output

- [ ] Client calls should configure timeouts
  - [x] - hollywood client request_timeout
  - [ ] - nats client initialize: max number of attempts to retry
        connecting to nats

- [x] Agent configure Actor as a pubsub subscriber
- [x] Client should be able to publish messages to a subject

### Maybe
- [ ] Where does Docker fit in?
- [ ] Swap serde_json w/ protobuf for HollywoodMsg's
- [ ] Worker pools
      - We'd need to change how Actors are instantiated (Use Arc, etc?)
      - Not sure this is worth it for now
- [ ] Metrics: tokio metrics or some other implementation
- [ ] How do we handle api/message versioning?