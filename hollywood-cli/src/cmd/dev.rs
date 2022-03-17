use clap::StructOpt;
use hollywood::config::{Actor, System};
use hollywood::env::{format_hollywood_system, format_hollywood_system_nats_uri};
use log::{info, warn};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::ops::Index;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(StructOpt, Debug)]
pub(crate) struct Opts {
    /// The name of the system to run
    #[clap(short, long)]
    system: String,

    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,
}

pub(crate) fn handle(opts: Opts) -> Result<(), Error> {
    // validate options...
    if opts.system == "" {
        return Err(Error::new(ErrorKind::InvalidInput, "missing --system"));
    }

    if opts.config.is_none() {
        return Err(Error::new(ErrorKind::InvalidInput, "missing --config"));
    }

    // Load the hollywood config...
    // build the config path from current dir...
    let dir = std::env::current_dir()?;

    let mut config_path = dir.clone();
    let config_file = opts.config.unwrap();
    config_path.push(&config_file);

    info!("load hollywood config from path {:?}", &config_path);
    let cfg = hollywood::config::Config::load(&config_path)?;
    info!("loaded hollywood config: {:#?}", &cfg);

    let index = &cfg.system.iter().position(|sys| &sys.name == &opts.system);
    if index.is_none() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "hollywood config has no system by the name of {:?}",
                &opts.system
            ),
        ));
    }

    let system = cfg.system.index(index.unwrap()).to_owned();
    if system.nats_uri == "" {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "system is missing a nats_uri",
        ));
    }

    info!("running system {:?}", &system);
    let mut rt = Runtime::new(dir, system, cfg.actor);
    rt.run()
}

fn validate_actors(actors: &Vec<hollywood::config::Actor>) -> Result<(), Error> {
    for actor in actors {
        if actor.dev.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "hollywood.toml [actor.dev] init",
            ));
        }
        let dev = &actor.dev.as_ref().unwrap();
        if dev.path == "" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "hollywood.toml [actor.dev.path] is empty",
            ));
        }
        if dev.bin == "" {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "hollywood.toml [actor.dev.bin] is empty",
            ));
        }
    }
    Ok(())
}

struct Runtime {
    dir: PathBuf,
    system: System,
    actor: Vec<Actor>,
}

impl Runtime {
    fn new(dir: PathBuf, system: System, actor: Vec<Actor>) -> Self {
        Self { dir, system, actor }
    }

    //fn run_actor(&mut self, actor: &Actor) {
    fn spawn(dir: PathBuf, system: System, actor: Actor) {
        // The watch command looks something like this...
        // `RUST_LOG=info cargo watch -w examples/types -x 'run --bin=actor-x'`

        info!("spawn {:?} for system: {:?}", &actor, &system);

        // TODO: Load actor Cargo.toml so we can get the package name?

        // get dev
        let dev = actor.dev.unwrap();

        fn prefix_watch_path(mut prefix: PathBuf, suffix: String) -> PathBuf {
            prefix.push(suffix);
            prefix
        }

        // init env vars
        let mut env: HashMap<&str, &str> = HashMap::new();
        for e in &dev.env {
            let mut parts = e.split("=");
            let key = parts.next();
            let val = parts.next();
            if key.is_none() || val.is_none() {
                warn!("invalid env variable: {}", &e);
                continue;
            }
            env.insert(&key.unwrap(), val.unwrap());
        }
        // add HOLLYWOOD env vars here
        let hollywood_system_env = format_hollywood_system();
        let hollywood_system_nats_uri_env = format_hollywood_system_nats_uri(system.name.clone());
        env.insert(&hollywood_system_env, &system.name);
        env.insert(&hollywood_system_nats_uri_env, &system.nats_uri);

        // init args
        let mut args = Vec::new();
        args.push("watch".to_owned());

        // watch args
        args.push("-w".to_owned());
        let fd = prefix_watch_path(dir.clone(), dev.path);
        if let Some(path) = fd.to_str() {
            args.push(path.to_owned());
        }

        for w in dev.watch {
            args.push("-w".to_owned());
            let fd = prefix_watch_path(dir.clone(), w);
            if let Some(path) = fd.to_str() {
                args.push(path.to_owned());
            }
        }

        // delay
        args.push("-d".to_owned());
        args.push("1".to_owned());

        // run args
        let exec = format!("run --bin {}", &dev.bin);
        args.push("-x".to_owned());
        args.push(exec);

        info!(
            "spawn {} cmd env: {:?} and args: {:?}",
            &actor.name, &env, &args
        );
        let _ = Command::new("cargo")
            .stdout(Stdio::inherit())
            .envs(&env)
            .args(&args)
            .spawn()
            .expect("failed to launch actor");

        info!("spawned {} thread", &actor.name);
    }

    fn run(&mut self) -> Result<(), Error> {
        // TODO: validate actors here...
        validate_actors(&self.actor)?;

        // iterate all actors here...
        // and run them
        for actor in &self.actor {
            let dir = self.dir.clone();
            let sys = self.system.clone();
            let act = actor.clone();
            let _ = thread::spawn(|| Runtime::spawn(dir, sys, act));
        }

        // add signal handler
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
