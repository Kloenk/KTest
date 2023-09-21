mod make;
pub use make::{Arch, Make};
mod qemu;
pub use qemu::{Qemu, QemuConfig};

use crate::Result;

use config::{ConfigError, Environment, File};
use serde_derive::Deserialize;
use tracing::*;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub qemu: qemu::Qemu,
    pub make: make::Make,
}

impl Config {
    const EMBED_CONFIG_STR: &'static str = include_str!("config.toml");

    pub fn new() -> Result<Self, ConfigError> {
        let cfg = config::Config::builder().add_source(File::from_str(
            Self::EMBED_CONFIG_STR,
            config::FileFormat::Toml,
        ));

        let cfg = if let Some(dist_file) = option_env!("KTEST_DIST_CONFIG_TOML") {
            cfg.add_source(config::File::new(dist_file, config::FileFormat::Toml))
        } else {
            cfg
        };

        let cfg = cfg
            .add_source(config::File::with_name("/etc/ktest/config").required(false))
            .add_source(config::File::with_name("ktest").required(false))
            .add_source(Environment::with_prefix("KTEST"))
            .build()?;

        cfg.try_deserialize()
    }

    pub fn init(&self) -> Result {
        // TODO: log init
        let logger = tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        // TODO: parse from config and command line

        logger.init();
        info!("Initialized config and logger");

        Ok(())
    }

    pub fn qemu_path(&self) -> Option<&str> {
        self.make.arch.as_ref().map(|a| self.qemu.path(a)).flatten()
    }

    pub fn qemu_args(&self) -> impl Iterator<Item = &str> {
        self.qemu.qemu_args(&self.make.arch.as_ref().unwrap())
    }

    pub fn qemu_kernel_args(&self) -> impl Iterator<Item = &str> {
        self.qemu.kernel_args(&self.make.arch.as_ref().unwrap())
    }
}
