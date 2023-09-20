use crate::config::Config;
use crate::make::MakeCmd;
use crate::Result;
use clap::{Arg, Command};
use tracing::*;

pub fn command(_config: &Config) -> Command {
    Command::new("make")
        .arg(
            Arg::new("make-command")
                .action(clap::ArgAction::Set)
                .value_parser(clap::value_parser!(String))
                .index(1),
        )
        .arg(
            Arg::new("make-args")
                .action(clap::ArgAction::Append)
                .value_parser(clap::value_parser!(String))
                .index(2)
                .trailing_var_arg(true),
        )
}

#[instrument(name = "make", level = "debug", skip(config, matches))]
pub fn run(config: &Config, matches: &clap::ArgMatches) -> Result {
    super::config::new_config(
        config,
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )?;

    let mut make = MakeCmd::new(
        config,
        matches
            .get_one::<String>("make-command")
            .map(|s| s.as_str()),
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )?;

    make.run()?;

    Ok(())
}
