use crate::config::Config;
use crate::make::MakeCmd;
use crate::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::*;

#[instrument(level = "trace", skip(config), fields(make = config.make.path.as_str()))]
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

    for (key, val) in &config.make.kconfig {
        set_config(config, &config_file, key, val)?;
    }

    Ok(config_file)
}

#[instrument(level = "debug", skip(config))]
pub fn set_config(config: &Config, file: &Path, key: &str, value: &str) -> Result {
    let mut ktool = PathBuf::from(&config.make.kernel_dir);
    ktool.push("scripts");
    ktool.push("config");

    debug!("Setting config option");
    let status = Command::new(ktool)
        .arg("--file")
        .arg(file)
        .arg("--set-val")
        .arg(key)
        .arg(value)
        .status()?;
    // TODO: propagate error?
    trace!("config status status: {}", status);

    Ok(())
}

pub fn check_configs(config: &Config, file: &Path) -> Result {
    for (key, val) in &config.make.kconfig {
        check_config(config, file, key, val)?;
    }
    info!("validated config");
    Ok(())
}

#[instrument(level = "debug", skip(config))]
pub fn check_config(config: &Config, file: &Path, key: &str, value: &str) -> Result {
    let mut ktool = PathBuf::from(&config.make.kernel_dir);
    ktool.push("scripts");
    ktool.push("config");

    let out = Command::new(ktool)
        .arg("--file")
        .arg(file)
        .arg("-s")
        .arg(key)
        .output()?;

    if !out.status.success() {
        info!("Failed to run config tool: {}", out.status);
        return Err(Error::new("Failed to run config tool").set_exit_code(out.status.code()));
    }

    let c = core::str::from_utf8(&out.stdout)?;
    let c = if c == "undef" { "n" } else { c }.trim();

    if c != value {
        return Err(
            Error::new(format!("Config mismatch: `{key}`: `{c}` != `{value}`"))
                .set_exit_code(Some(1)),
        );
    }

    Ok(())
}

pub fn parse(str: &str) -> Result<(String, String)> {
    let parts = str.split("=").collect::<Vec<_>>();
    match parts.len() {
        1 => Ok((parts[0].to_string(), "y".to_string())),
        2 => Ok((parts[0].to_string(), parts[1].to_string())),
        _ => Err(Error::new(format!("Invalid config option: {}", str))),
    }
}
