use crate::config::Config;
use crate::make::MakeCmd;
use anyhow::{bail, Context, Result};
use tracing::*;

pub fn command(_config: &Config) -> clap::Command {
    clap::Command::new("oldconfig")
        .about("Run make oldconfig")
        .arg(
            clap::Arg::new("make-args")
                .action(clap::ArgAction::Append)
                .value_parser(clap::value_parser!(String))
                .trailing_var_arg(true),
        )
}

#[instrument(name = "oldconfig", level = "debug", skip(config, matches))]
pub async fn run(config: &Config, matches: &clap::ArgMatches) -> Result<()> {
    super::config::new_config(
        config,
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )
    .await?;

    let mut make = MakeCmd::new(
        config,
        Some("oldconfig"),
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )
    .await?;

    let status = make.cmd.status().await?;
    trace!("make oldconfig exited with status: {}", status);
    if !status.success() {
        bail!("Failed to run oldconfig: {}", status);
    }

    Ok(())
}
