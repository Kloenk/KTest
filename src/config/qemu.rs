use clap::{Arg, ArgAction, ArgMatches, Command, Error, FromArgMatches, ValueHint};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Qemu {
    pub path_override: Option<String>,
    #[serde(flatten)]
    pub arch_config: HashMap<super::make::Arch, QemuConfig>,
}

impl Qemu {
    pub fn path(&self, arch: &super::make::Arch) -> Option<&str> {
        self.path_override
            .as_ref()
            .or_else(|| self.arch_config.get(arch).map(|c| &c.path))
            .map(String::as_str)
    }

    pub fn augument_args(&self, cmd: Command) -> Command {
        cmd.arg(
            Arg::new("qemu-path")
                .long("qemu-path")
                .action(ArgAction::Set)
                .value_name("EXECUTABLE")
                .value_hint(ValueHint::ExecutablePath)
                .hide(true),
        )
        .group(
            clap::ArgGroup::new("qemu-args")
                .args(["qemu-path"])
                .multiple(true),
        )
    }

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

impl FromArgMatches for Qemu {
    fn from_arg_matches(_matches: &ArgMatches) -> Result<Self, Error> {
        unreachable!()
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        self.path_override = matches.get_one::<String>("qemu-path").map(String::clone);

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QemuConfig {
    pub path: String,
    //pub args: Vec<String>,
}
