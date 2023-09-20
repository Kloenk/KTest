use crate::config::Config;
use crate::{Context, Error, Result};
use std::process::Command;

pub struct QemuCmd {
    pub cmd: Command,
}

impl QemuCmd {
    pub fn new(config: &Config) -> Result<Self> {
        let mut cmd = Command::new(config.qemu_path().context("Missing qemu executable")?);

        todo!()
    }
}
