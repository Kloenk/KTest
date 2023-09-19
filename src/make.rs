use std::ffi::OsStr;
use std::process::Command;

use crate::config::Config;
use crate::{Context, Error, Result};
use tracing::*;

pub fn create_jobserver(config: &Config) -> Result<jobserver::Client> {
    let jobs = config.make.jobs.unwrap();

    debug!("Creating jobserver with {jobs} jobs");
    jobserver::Client::new(jobs).context("Failed to create jobserver")
}

pub struct MakeCmd {
    pub cmd: tokio::process::Command,
    pub jobserver: jobserver::Client,
}

impl MakeCmd {
    pub async fn new<I, S>(config: &Config, command: Option<&str>, args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let jobserver = create_jobserver(config)?;

        let mut cmd = Command::new(&config.make.path);
        cmd.current_dir(
            std::path::Path::new(&config.make.kernel_dir)
                .canonicalize()
                .context("Failed to resolve kernel source directory")?,
        );
        jobserver.configure_make(&mut cmd);
        let mut cmd = tokio::process::Command::from(cmd);
        cmd.arg(config.make.make_arch_arg());
        cmd.arg(config.make.make_build_dir_arg());
        cmd.arg(format!(
            "INSTALL_MOD_PATH={}",
            config
                .make
                .kernel_bin_dir()
                .to_str()
                .context("Invalid kernel bin dir")?
        ));
        cmd.args(&config.make.extra_make_args);
        if let Some(command) = command {
            cmd.arg(command);
        }
        cmd.args(args);

        Ok(Self { cmd, jobserver })
    }

    pub async fn run(&mut self) -> Result {
        let status = self.cmd.status().await.context("Error executing make")?;

        if !status.success() {
            info!("Failed to run make: {}", status);
            Err(Error::new("Failed to run command").set_exit_code(status.code()))
        } else {
            Ok(())
        }
    }
}
