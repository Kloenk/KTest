mod make;

use anyhow::{bail, Context, Result};
use std::str::FromStr;
use std::{env, ffi::OsString};

use clap::builder::PossibleValue;
use clap::{
    builder, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, FromArgMatches, ValueEnum,
    ValueHint,
};
use config::{ConfigError, Environment, File};
use log::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Qemu {
    pub path: String,
}

impl Qemu {
    pub fn args(&self) -> Vec<Arg> {
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
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rt {
    pub workers: Option<usize>,
    pub max_blocking: Option<usize>,
    pub stack_size: Option<usize>,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub qemu: Qemu,
    pub make: make::Make,
    tokio_runtime: Rt,
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

    pub fn init(&self) -> Result<tokio::runtime::Runtime> {
        // TODO: log init

        let rt = self.build_runtime()?;
        info!("Initialized config");

        Ok(rt)
    }

    pub fn build_runtime(&self) -> Result<tokio::runtime::Runtime> {
        use tokio::runtime::Builder;
        let mut builder = Builder::new_multi_thread();
        builder
            .enable_all()
            .thread_name(self.tokio_runtime.name.as_str());

        if let Some(max_blocking) = self.tokio_runtime.max_blocking {
            builder.max_blocking_threads(max_blocking);
        }
        if let Some(workers) = self.tokio_runtime.workers {
            builder.worker_threads(workers);
        }
        if let Some(stack) = self.tokio_runtime.stack_size {
            builder.thread_stack_size(stack);
        }

        Ok(builder.build()?)
    }
}
