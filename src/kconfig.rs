use crate::config::Config;
use crate::make::MakeCmd;
use crate::{Error, Result};
use ktest_core::config::KConfig;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
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

    for cfg in config.make.kconfig.values() {
        set_config(config, &config_file, cfg)?;
    }

    Ok(config_file)
}

#[instrument(level = "debug", skip(config))]
pub fn set_config(config: &Config, file: &Path, cfg: &KConfig) -> Result {
    let mut ktool = PathBuf::from(&config.make.kernel_dir);
    ktool.push("scripts");
    ktool.push("config");

    debug!("Setting config option");
    let status = Command::new(ktool)
        .arg("--file")
        .arg(file)
        .arg("--set-val")
        .arg(&cfg.key)
        .arg(cfg.value.as_deref().unwrap_or("y"))
        .status()?;
    // TODO: propagate error?
    trace!("config status status: {}", status);

    Ok(())
}

pub fn check_configs(config: &Config, file: &Path) -> Result {
    for cfg in config.make.kconfig.values() {
        check_config(config, file, cfg)?;
    }
    info!("validated config");
    Ok(())
}

#[instrument(level = "debug", skip(config))]
pub fn check_config(config: &Config, file: &Path, cfg: &KConfig) -> Result {
    let mut ktool = PathBuf::from(&config.make.kernel_dir);
    ktool.push("scripts");
    ktool.push("config");

    let out = Command::new(ktool)
        .arg("--file")
        .arg(file)
        .arg("-s")
        .arg(&cfg.key)
        .output()?;

    if !out.status.success() {
        info!("Failed to run config tool: {}", out.status);
        return Err(Error::new("Failed to run config tool").set_exit_code(out.status.code()));
    }

    let c = core::str::from_utf8(&out.stdout)?;
    let c = if c == "undef" { "n" } else { c }.trim();

    if c != cfg.value.as_deref().unwrap_or("y") {
        return Err(Error::new(format!(
            "Config mismatch: `{key}`: `{c}` != `{value}`",
            key = cfg.key,
            c = c,
            value = cfg.value.as_deref().unwrap_or("y")
        ))
        .set_exit_code(Some(1)));
    }

    Ok(())
}
