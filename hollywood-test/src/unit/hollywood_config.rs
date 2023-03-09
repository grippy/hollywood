#[test]
fn test_parser() {
    use hollywood::config::{Actor, ActorDev, Config, System};
    use toml;

    let cfg = Config {
        system: vec![
            System {
                name: "system1".into(),
                nats_uri: "nats://system1".into(),
            },
            System {
                name: "system2".into(),
                nats_uri: "nats://system2".into(),
            },
        ],
        actor: vec![
            Actor {
                name: "Actor1".into(),
                dev: Some(ActorDev {
                    path: "examples/actor-1".into(),
                    bin: "actor-1".into(),
                    env: vec!["ENV1=env1".into()],
                    watch: vec!["dep1".into(), "dep2".into()],
                }),
                test: None,
            },
            Actor {
                name: "Actor2".into(),
                dev: Some(ActorDev {
                    path: "examples/actor-2".into(),
                    bin: "actor-2".into(),
                    env: vec!["ENV2=env2".into()],
                    watch: vec!["dep2".into(), "dep3".into()],
                }),
                test: None,
            },
        ],
    };

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

    assert_eq!(config, cfg);
}
