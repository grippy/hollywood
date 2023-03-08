#[test]
fn test_hollywood_system_env() {
    use hollywood::env::{hollywood_system, set_hollywood_system};
    let system = "test".to_owned();
    set_hollywood_system(system.clone());
    assert_eq!(hollywood_system().is_err(), false);
    assert_eq!(system, hollywood_system().unwrap());
}

#[test]
fn test_hollywood_system_nats_uri_env() {
    use hollywood::env::{hollywood_system_nats_uri, set_hollywood_system_nats_uri};
    let system = "test".to_owned();
    let nats_uri = "nats://127.0.0.1".to_owned();
    set_hollywood_system_nats_uri(system.clone(), nats_uri.clone());
    assert_eq!(hollywood_system_nats_uri(system.clone()).is_err(), false);
    assert_eq!(nats_uri, hollywood_system_nats_uri(system.clone()).unwrap());
}
