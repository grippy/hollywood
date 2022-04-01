use anyhow::Result;
use std::env;

static HOLLYWOOD_SYSTEM: &'static str = "HOLLYWOOD_SYSTEM";

fn get(var: String) -> Result<String> {
    match env::var(var) {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into()),
    }
}

/// formats `HOLLYWOOD_SYSTEM` env variable as string
pub fn format_hollywood_system() -> String {
    format!("{}", HOLLYWOOD_SYSTEM)
}

/// formats `HOLLYWOOD_SYSTEM_{SYSTEM_NAME}_NATS_URI` env variable
pub fn format_hollywood_system_nats_uri(system_name: String) -> String {
    format!("HOLLYWOOD_SYSTEM_{}_NATS_URI", system_name.to_uppercase())
}

// Returns the HOLLYWOOD_SYSTEM env variable
pub fn hollywood_system() -> Result<String> {
    get(HOLLYWOOD_SYSTEM.to_owned())
}

// Sets the HOLLYWOOD_SYSTEM env variable
pub fn set_hollywood_system(system_name: String) {
    env::set_var(HOLLYWOOD_SYSTEM, system_name);
}

/// Returns the Nats uri env variable for a given HOLLYWOOD_SYSTEM
/// env var is in this format: HOLLYWOOD_SYSTEM_{SYSTEM_NAME}_NATS_URI
pub fn hollywood_system_nats_uri(system_name: String) -> Result<String> {
    let var = format_hollywood_system_nats_uri(system_name);
    get(var)
}

// Sets the Nats uri for the given system
pub fn set_hollywood_system_nats_uri(system_name: String, nats_uri: String) {
    let var = format_hollywood_system_nats_uri(system_name);
    env::set_var(var, nats_uri);
}
