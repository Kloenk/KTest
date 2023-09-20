mod make;
mod qemu;

use crate::Result;

use config::{ConfigError, Environment, File};
use serde_derive::Deserialize;
use tracing::*;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub qemu: qemu::Qemu,
    pub make: make::Make,
}

impl Config {
    const EMBED_CONFIG_STR: &'static str = include_str!("config.toml");

    pub fn new() -> Result<Self, ConfigError> {
        let cfg = config::Config::builder()
            .add_source(File::from_str(
                Self::EMBED_CONFIG_STR,
                config::FileFormat::Toml,
            ))
            .add_source(Environment::with_prefix("KTEST"))
            .build()?;

        cfg.try_deserialize()
    }

    pub fn init(&self) -> Result {
        // TODO: log init
        let logger = tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env());
        // TODO: parse from config and command line

        logger.init();
        info!("Initialized config and logger");

        Ok(())
    }

    pub fn qemu_path(&self) -> Option<&str> {
        self.make.arch.as_ref().map(|a| self.qemu.path(a)).flatten()
    }
}
