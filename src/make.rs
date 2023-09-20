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
    pub cmd: Command,
    pub jobserver: jobserver::Client,
}

impl MakeCmd {
    pub fn new<I, S>(config: &Config, command: Option<&str>, args: I) -> Result<Self>
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

    pub fn run(&mut self) -> Result {
        debug!(
            "Running {} {}",
            self.cmd
                .get_program()
                .to_str()
                .unwrap_or("**could not find executable**"),
            self.cmd
                .get_args()
                .filter_map(|a| a.to_str())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let status = self.cmd.status().context("Error executing make")?;

        if !status.success() {
            info!("Failed to run make: {}", status);
            Err(Error::new("Failed to run make").set_exit_code(status.code()))
        } else {
            Ok(())
        }
    }
}
