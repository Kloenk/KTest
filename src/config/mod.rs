mod make;

use crate::Result;

use config::{ConfigError, Environment, File};
use serde_derive::Deserialize;
use tracing::*;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Deserialize)]
pub struct Qemu {
    pub path: String,
}

impl Qemu {
    /*pub fn args(&self) -> Vec<Arg> {
        let mut ret = Vec::new();

        ret.push(
            Arg::new("qemu-path")
                .long("qemu-path")
                .action(ArgAction::Set)
                .value_name("PATH")
                .value_hint(ValueHint::ExecutablePath)
                .default_value(self.path.clone())
                .hide(true),
        );

        ret
    }

    pub fn group() -> ArgGroup {
        ArgGroup::new("qemu")
            //.args(["qemu-path"])
            .multiple(true)
    }*/
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub qemu: Qemu,
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
}
