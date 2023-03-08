// This defines the main hollywood-cli
// application.
mod cmd;

use clap::{Parser, Subcommand};
use log::info;
use pretty_env_logger;
use std::io::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct HollywoodCli {
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Builds and runs actors using `cargo watch`
    Dev(cmd::dev::Opts),

    /// Builds and runs actors using `cargo build & cargo run`
    Test(cmd::test::Opts),
}

fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let cli = HollywoodCli::parse();
    if cli.cmd.is_some() {
        let cmd = cli.cmd.unwrap();
        info!("running cmd: {:#?}", &cmd);
        match cmd {
            Cmd::Dev(opts) => cmd::dev::handle(opts),
            Cmd::Test(opts) => cmd::test::handle(opts),
        }?
    }

    Ok(())
}
