## TODO

- [ ] Test coverage is non-existent
- [ ] Health checks
- [ ] Shutdown hooks
- [x] hollywood-cli dev should check if `cargo watch` is installed
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
- [ ] Mailbox that is specific for Actor version + Msg version
      (so we don't have to pass type hints with mailbox calls).

      mailbox::actor1_v1::msg1_v1_0::new(...)
      mailbox::actor1_v1::msg1_v1_1::new(...)
      mailbox::actor2_v1::msg2_v1_0::new(...)
      mailbox::actor2_v1::msg2_v1_1::new(...)

      Maybe we could add this to the Hollywood macro so it produces
      a mod for the actor version and msg version.

- [ ] Client calls should configure timeouts
  - [x] - hollywood client request_timeout
  - [ ] - nats client initialize: max number of attempts to retry
        connecting to nats
- [ ] Nats clients should retry connecting if nats server goes away
- [x] Agent configure Actor as a pubsub subscriber
- [x] Client should be able to publish messages to a subject
- [x] Versioned Actor messages
    - [x] Dispatch trait + macro
    - [x] rework Client and make Mailbox wrapper
- [ ] Examples: redis client should support retries
- [ ] Nats: support JetStream as a Queue?

### Maybe
- [ ] Where does Docker fit in?
- [ ] Swap serde_json w/ protobuf for HollywoodMsg's
- [ ] Worker pools
      - We'd need to change how Actors are instantiated (Use Arc, etc?)
      - Not sure this is worth it for now
- [ ] Metrics: tokio metrics or some other implementation
