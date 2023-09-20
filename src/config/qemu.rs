use clap::{value_parser, Arg, ArgAction, ArgMatches, Command, Error, FromArgMatches, ValueHint};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Qemu {
    pub path_override: Option<String>,
    pub extra_args: Vec<String>,
    pub extra_kernel_args: Vec<String>,
    #[serde(flatten)]
    pub arch_config: HashMap<super::make::Arch, QemuConfig>,

    #[serde(default)]
    pub storage_bus: String,
    #[serde(default = "default_mem")]
    pub mem: String,
    #[serde(default)]
    pub cpus: usize,
}

impl Qemu {
    pub fn path(&self, arch: &super::make::Arch) -> Option<&str> {
        self.path_override
            .as_ref()
            .or_else(|| self.arch_config.get(arch).map(|c| &c.path))
            .map(String::as_str)
    }

    pub fn qemu_args(&self, arch: &super::make::Arch) -> impl Iterator<Item = &str> {
        self.extra_args
            .iter()
            .chain(
                self.arch_config
                    .get(arch)
                    .map(|c| c.args.iter())
                    .unwrap_or_default(),
            )
            .map(String::as_str)
    }

    pub fn kernel_args(&self, arch: &super::make::Arch) -> impl Iterator<Item = &str> {
        self.extra_kernel_args
            .iter()
            .chain(
                self.arch_config
                    .get(arch)
                    .map(|c| c.kernel_args.iter())
                    .unwrap_or_default(),
            )
            .map(String::as_str)
    }

    pub fn mem(&self) -> &str {
        self.mem.as_str()
    }

    pub fn cpus(&self) -> usize {
        self.cpus
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
        .arg(
            Arg::new("qemu-extra-args")
                .long("qemu-extra-args")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("qemu-extra-kernel-args")
                .long("extra-kernel-args")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("qemu-mem")
                .long("memory")
                .action(ArgAction::Set)
                .value_parser(value_parser!(String))
                .value_name("SIZE")
                .hide(true)
                .default_value(&self.mem),
        )
        .arg(
            Arg::new("qemu-cpus")
                .long("cpus")
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize))
                .value_name("NUM")
                .hide(true)
                .default_value("1"),
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
        self.mem = matches
            .get_one::<String>("qemu-mem")
            .map(String::clone)
            .unwrap();
        self.cpus = matches.get_one::<usize>("qemu-cpus").map(|v| *v).unwrap();

        for arg in matches
            .get_many::<String>("qemu-extra-args")
            .unwrap_or_default()
        {
            self.extra_args.push(arg.clone());
        }
        for arg in matches
            .get_many::<String>("qemu-extra-kernel-args")
            .unwrap_or_default()
        {
            self.extra_kernel_args.push(arg.clone());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QemuConfig {
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub kernel_args: Vec<String>,
}

fn default_mem() -> String {
    "1G".to_string()
}
