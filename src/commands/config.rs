use crate::config::Config;
use crate::make::MakeCmd;
use crate::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::*;

pub fn command(_config: &Config) -> clap::Command {
    clap::Command::new("config")
        .about("Configure kernel running nconfig")
        .arg(
            clap::Arg::new("make-args")
                .action(clap::ArgAction::Append)
                .value_parser(clap::value_parser!(String))
                .trailing_var_arg(true),
        )
}

#[tracing::instrument(name = "config", level = "debug", skip(config, matches))]
pub fn run(config: &Config, matches: &clap::ArgMatches) -> Result {
    new_config(
        config,
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )?;

    MakeCmd::new(
        config,
        Some("nconfig"),
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )?
    .run()?;

    Ok(())
}

#[tracing::instrument(level = "trace", skip(config), fields(make = config.make.path.as_str()))]
pub fn new_config<I, S>(config: &Config, args: I) -> Result<PathBuf>
where
    I: IntoIterator<Item = S> + core::fmt::Debug,
    S: AsRef<std::ffi::OsStr>,
{
    let mut config_file = config.make.make_build_dir();
    config_file.push(".config");

    if !config_file.exists() {
        debug!("Running allnoconfig");
        MakeCmd::new(config, Some("allnoconfig"), args)?.run()?;

        debug!("Clear full config");
        let status = Command::new("sed")
            .arg("-i")
            .arg("-e")
            .arg("s/\\(CONFIG_.*\\)=.*/# \\1 is not set/")
            .arg(config_file.as_os_str())
            .status()?;
        trace!("sed status: {}", status);

        // TODO: replace all with n
    }

    Ok(config_file)
}
