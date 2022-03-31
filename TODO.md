## TODO

- [ ] Test coverage is non-existent
- [ ] Health checks
- [ ] Shutdown hooks
- [ ] hollywood-cli dev should check if `cargo watch` is installed
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
- [ ] Nats clients should retry connecting if nats server goes away

- [x] Agent configure Actor as a pubsub subscriber
- [x] Client should be able to publish messages to a subject

- [x] Versioned Actor messages
    - [x] Dispatch trait + macro
    - [x] rework Client and make Mailbox wrapper

### Maybe
- [ ] Where does Docker fit in?
- [ ] Swap serde_json w/ protobuf for HollywoodMsg's
- [ ] Worker pools
      - We'd need to change how Actors are instantiated (Use Arc, etc?)
      - Not sure this is worth it for now
- [ ] Metrics: tokio metrics or some other implementation
