# TODO

Things to work on...

## Alpha v0.1

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
- [x] Agent configure Actor as a pubsub subscriber
- [x] Client request_timeout
- [x] Client should be able to publish messages to a subject
- [x] Versioned Actor messages
    - [x] Dispatch trait + macro
    - [x] rework Client and make Mailbox wrapper

## Alpha v0.2

- [x] hollywood-cli should use `hollywood.toml` as the default config file
- [ ] Test coverage is non-existent
    - [ ] unit, integration tests, etc.
    - [ ] Docker/Docker-compose test environment

- [ ] Broker should have shutdown hooks
    - [ ] Broker inbox should read hollywood system messages
          and respond to shutdown/health, etc.
    - [ ] signal hooks?
- [ ] `hollywood-cli test` which launches the compiled actors
       using one process per actor

## Beyond

- [ ] Health checks
- [ ] Shutdown hooks
- [ ] Client calls should configure timeouts
  - [ ] - nats client initialize: max number of attempts to retry
        connecting to nats
- [ ] Nats clients should retry connecting if nats server goes away
- [ ] Examples: redis client should support retries
- [ ] Nats: support JetStream as a Queue?
- [ ] Hollywood macro: dispatch types shouldn't be namespaced:
        - So, `#[dispatch(module1::Msg)]` will throw an error
          but `#[dispatch(Msg)]` doesn't
- [ ] Hollywood macro: can we provide default implementations
      for the Handler<T> trait implementations? So, instead of having to
      implement send, request, and subscribe... the handler only
      implements 1 method and the others are automatically filled in.
- [ ] Mailbox that is specific for Actor version + Msg version
      (so we don't have to pass type hints with mailbox calls).

      ```
      mailbox::actor1_v1::msg1_v1_0::new(...)
      mailbox::actor1_v1::msg1_v1_1::new(...)
      mailbox::actor2_v1::msg2_v1_0::new(...)
      mailbox::actor2_v1::msg2_v1_1::new(...)
      ```

      Maybe we could add this to the Hollywood macro so it produces
      a module for the actor > version > msg > version?

### Maybe
- [ ] Worker pools
      - We'd need to change how Actors are instantiated (Use Arc, etc?)
      - Not sure this is worth it for now
- [ ] Tracing/Metrics: tokio metrics or some other implementation
