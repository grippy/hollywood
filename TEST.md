# Ideas

- Hollywood example testing framework
- Actor testing framework

# Challenges
1. How do we test actors that might call other actors without running them as a system?
    - instead of using the actor client, we instantiate the actor and just call it?

2. Should actors define dependencies with other actors?
    - Dependency: Actor1, Actor2

3. Unit vs Integration, what's the difference?

4. Can we use the hollywood.toml to define how/what to test?

# Harness

```
hollywood-cli test
    --unit
    --integration
    --e2e --system=X --config=hollywood.toml
```

## Unit Testing
- use cargo test?

## Integration Testing
- use cargo test?

## End-to-End Testing
- Build actors
- Setup, teardown test environment
- Run system command
    - Docker-compose cmd
- End-to-end tests call actors with messages
    and assert results
    - What assertions are made for send/publish calls?
