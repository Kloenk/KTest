use clap::{Arg, FromArgMatches};
use tracing::trace;

mod boot;
mod build;
mod commands;
mod config;
mod err;
mod kconfig;
mod make;

pub use err::{Context, Error, Result};

pub fn main() {
    let config = config::Config::new().expect("Failed to read config");
    config.init().expect("Failed to initialize async runtime");

    if let Err(e) = run_main(config) {
        trace!("Failed to run ktest: {e:?}");
        e.exit();
    }
}

fn run_main(mut config: config::Config) -> Result {
    let app = clap::command!()
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        )
        .subcommand_required(true)
        .subcommand(commands::make::command(&config))
        .subcommand(commands::config::command(&config))
        .subcommand(commands::oldconfig::command(&config))
        .subcommand(commands::build::command(&config))
        .subcommand(commands::boot::command(&config))
        .subcommand(commands::run::command(&config));
    let app = config.make.augument_args(app);

    let matches = app.get_matches();

    config
        .make
        .update_from_arg_matches(&matches)
        .context("Failed to parse matches")?;
    trace!("Loaded config: {config:?}");

    match matches.subcommand().context("No subcomand provided")? {
        ("make", matches) => commands::make::run(&mut config, &matches)?,
        ("config", matches) => commands::config::run(&mut config, &matches)?,
        ("oldconfig", matches) => commands::oldconfig::run(&mut config, &matches)?,
        ("build", matches) => commands::build::run(&mut config, &matches)?,
        ("boot", matches) => commands::boot::run(&mut config, &matches)?,
        ("run", matches) => commands::run::run(&mut config, &matches)?,

        _ => return Err(Error::new("Unknown subcommand")),
    };
    Ok(())
}
