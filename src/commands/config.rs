use crate::config::Config;
use crate::make::MakeCmd;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

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

pub async fn run(config: &Config, matches: &clap::ArgMatches) -> Result<()> {
    new_config(
        config,
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )
    .await?;

    let mut make = MakeCmd::new(
        config,
        Some("nconfig"),
        matches.get_many::<String>("make-args").unwrap_or_default(),
    )
    .await?;

    let status = make.cmd.status().await?;

    println!("Exited with status: {}", status);

    Ok(())
}

pub async fn new_config<I, S>(config: &Config, args: I) -> Result<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut config_file = config.make.make_build_dir();
    config_file.push(".config");

    if !config_file.exists() {
        let mut make = MakeCmd::new(config, Some("allnoconfig"), args).await?;

        let status = make.cmd.status().await?;
        if !status.success() {
            bail!("Failed to run allnoconfig: {}", status);
        }

        let cmd = tokio::process::Command::new("sed")
            .arg("-i")
            .arg("-e")
            .arg("s/\\(CONFIG_.*\\)=.*/# \\1 is not set/")
            .arg(config_file.as_os_str())
            .status()
            .await?;
        // TODO: replace all with n
    }

    Ok(config_file)
}
