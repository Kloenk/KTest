use crate::config::Config;
use crate::make::MakeCmd;
use crate::Result;
use tracing::*;

pub fn command(_config: &Config) -> clap::Command {
    clap::Command::new("build").about("Build the kernel").arg(
        clap::Arg::new("make-args")
            .long("args")
            .action(clap::ArgAction::Append)
            .value_parser(clap::value_parser!(String)),
    )
}

#[instrument(name = "build", level = "debug", skip(config, matches))]
pub fn run(config: &Config, matches: &clap::ArgMatches) -> Result {
    let args = matches.get_many::<String>("make-args").unwrap_or_default();

    crate::build::build(config, args)?;

    Ok(())
}
