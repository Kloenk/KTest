use std::process::Command;

use clap::{Arg, ArgAction, ArgGroup, ValueHint, ArgMatches};
use anyhow::{Result, Context};
use tracing::*;

pub fn common_args(config: &crate::config::Config) -> Vec<Arg> {
    let mut ret = Vec::new();

    /*ret.push(
        Arg::new("jobs")
            .action(ArgAction::Set)
            .short('j')
            .long("jobs")
            .value_name("JOBS")
            .value_parser(clap::value_parser!(u16)), // default nprocs
    );*/

    ret.push(
        Arg::new("build-dir")
            .action(ArgAction::Set)
            .short('b')
            .long("build")
            .value_hint(ValueHint::DirPath)
            .value_name("DIRECTORY")
            .value_parser(clap::value_parser!(String))
            .default_value(".ktest/build"),
    );

    ret.extend(config.make.args());
    ret.extend(config.qemu.args());

    ret
}

pub fn common_groups() -> Vec<ArgGroup> {
    let mut ret = Vec::new();

    ret.push(
        ArgGroup::new("common-make")
            //.args(["jobs", "build-dir"])
            .multiple(true),
    );
    ret.push(crate::config::Qemu::group());

    ret
}

pub fn create_jobserver(matches: &ArgMatches) -> Result<jobserver::Client> {
    // TODO: use nprocs
    let jobs = matches.get_one("jobs").map(|v| *v).unwrap_or_else(|| 1u16) as usize;

    debug!("Creating jobserver with {jobs} jobs");
    jobserver::Client::new(jobs).context("Failed to create jobserver")
}

pub fn make(matches: &ArgMatches) -> Result<String> {
    let jobserver = create_jobserver(matches)?;

    let make_path: &String = matches.get_one("make-path").context("make path was not set")?;

    let mut cmd = Command::new(make_path);

    jobserver.configure_make(&mut cmd);

    cmd.spawn()

    tokio
    todo!()
}