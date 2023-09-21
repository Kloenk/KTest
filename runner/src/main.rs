use ktest_core::config::Config;
use ktest_core::{Error, Result};
use tracing::*;

pub mod commands;

fn main() {
    let config = Config::new().expect("Failed to read config");
    config.init().expect("Failed to initialize async runtime");

    if let Err(e) = run_main(config) {
        trace!("Failed to run ktest: {e:?}");
        e.exit();
    }
}

fn run_main(mut config: Config) -> Result {
    let app = clap::command!().subcommand(commands::deps::command(&config));

    let matches = app.get_matches();

    if matches.subcommand().is_none() {
        todo!();
        return Ok(());
    }

    match matches.subcommand().unwrap() {
        ("deps", matches) => commands::deps::run(&mut config, &matches)?,
        //Some()
        //None => (),
        _ => return Err(Error::new("Unknown subcommand")),
    }

    Ok(())
}
