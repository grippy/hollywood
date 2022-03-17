use anyhow::Result;
use local_ip_address::local_ip;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub(crate) fn new_id() -> Uuid {
    let id = Uuid::new_v4();
    id
}

pub(crate) fn new_id_as_string() -> String {
    format!("{}", new_id())
}

#[allow(dead_code)]
pub(crate) fn now_as_duration() -> Duration {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[allow(dead_code)]
pub(crate) fn epoch_as_secs() -> i64 {
    now_as_duration().as_secs() as i64
}

#[allow(dead_code)]
pub(crate) fn local_ip_addr() -> Result<String> {
    match local_ip() {
        Ok(ip) => Ok(ip.to_string()),
        Err(err) => Err(err.into()),
    }
}