use crate::{Context, Error, Result};
use clap::builder::PossibleValue;
use clap::{value_parser, Arg, ArgAction, ArgMatches, FromArgMatches, ValueEnum, ValueHint};
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum Arch {
    X86,
    X86_64,
    Aarch64,
    Mips,
    Mips64,
    Sparc,
    Sparc64,
    PowerPC,
    PowerPC64,
}

impl Arch {
    pub fn kernel_arch(&self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86",
            Self::Aarch64 => "arm64",
            Self::Mips => "mips",
            Self::Mips64 => "mips",
            Self::Sparc => "sparc",
            Self::Sparc64 => "sparc",
            Self::PowerPC => "powerpc",
            Self::PowerPC64 => "powerpc",
        }
    }
}

impl core::fmt::Display for Arch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl std::str::FromStr for Arch {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(Error::new(format!("Invalid architecture: {}", s)))
    }
}

impl ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::X86,
            Self::X86_64,
            Self::Aarch64,
            Self::Mips,
            Self::Mips64,
            Self::Sparc,
            Self::Sparc64,
            Self::PowerPC,
            Self::PowerPC64,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::X86 => PossibleValue::new("x86").alias("i386"),
            Self::X86_64 => PossibleValue::new("x86_64").alias("amd64"),
            Self::Aarch64 => PossibleValue::new("aarch64").alias("arm64"),
            Self::Mips => PossibleValue::new("mips"),
            Self::Mips64 => PossibleValue::new("mips64"),
            Self::Sparc => PossibleValue::new("sparc"),
            Self::Sparc64 => PossibleValue::new("sparc64"),
            Self::PowerPC => PossibleValue::new("powerpc").alias("ppc"),
            Self::PowerPC64 => PossibleValue::new("powerpc64").alias("ppc64"),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Make {
    pub path: String,
    pub jobs: Option<usize>,
    pub arch: Option<Arch>,
    pub out_dir: String,
    pub kernel_dir: String,
    pub extra_make_args: Vec<String>,
}

impl Make {
    pub fn args(&self) -> Vec<Arg> {
        let mut ret = Vec::new();

        ret.push(
            Arg::new("make-path")
                .long("make-path")
                .action(ArgAction::Set)
                .value_name("PATH")
                .value_hint(ValueHint::ExecutablePath)
                .default_value(self.path.clone())
                .hide(true)
                .global(true),
        );

        let jobs = Arg::new("make-jobs")
            .short('j')
            .long("jobs")
            .value_parser(value_parser!(usize))
            .value_name("NUM")
            .hide_default_value(true)
            .global(true);
        let jobs = if let Some(num) = self.jobs {
            jobs.default_value(num.to_string())
        } else {
            jobs
        };
        ret.push(jobs);

        let arch = Arg::new("make-arch")
            .long("arch")
            .value_name("ARCH")
            .value_parser(clap::builder::EnumValueParser::<Arch>::new())
            .global(true)
            .hide_possible_values(true);
        let arch = if let Some(def) = self.arch {
            arch.default_value(def.to_string())
        } else {
            arch
        };
        ret.push(arch);

        ret.push(
            Arg::new("make-out-dir")
                .long("out-dir")
                .short('o')
                .value_name("DIR")
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::DirPath)
                .default_value(self.out_dir.clone())
                .global(true),
        );
        ret.push(
            Arg::new("make-kernel-dir")
                .long("kernel-source")
                .short('k')
                .value_name("DIR")
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::DirPath)
                .default_value(self.kernel_dir.clone())
                .global(true),
        );

        ret
    }

    pub fn kernel_bin_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.out_dir);
        path.push(format!(
            "kernel.{}",
            self.arch.as_ref().unwrap().kernel_arch()
        ));
        path
    }

    pub fn make_build_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.out_dir);
        path.push(format!(
            "kernel_build.{}",
            self.arch.as_ref().unwrap().kernel_arch()
        ));
        path
    }

    pub fn make_build_dir_arg(&self) -> String {
        format!("O={}", self.make_build_dir().to_str().unwrap())
    }

    pub fn make_arch_arg(&self) -> String {
        format!("ARCH={}", self.arch.as_ref().unwrap().kernel_arch())
    }

    fn jobs_or_default(matches: &ArgMatches) -> Result<usize> {
        let jobs = matches.get_one::<usize>("make-jobs").map(|v| *v);
        if let Some(jobs) = jobs {
            Ok(jobs)
        } else {
            get_nprocs().context("Failed to get number of processors")
        }
    }

    fn arch_or_default(matches: &ArgMatches) -> Result<Arch> {
        let arch = matches.get_one::<Arch>("make-arch").map(|v| *v);
        if let Some(arch) = arch {
            Ok(arch)
        } else {
            get_arch().context("Failed to get architecture")
        }
    }
}

impl FromArgMatches for Make {
    fn from_arg_matches(
        matches: &ArgMatches,
    ) -> Result<Self, clap::error::Error<clap::error::RichFormatter>> {
        tracing::warn!("Creating Make config without reading config");
        let ret = Self {
            path: matches.get_one::<String>("make-path").unwrap().clone(),
            jobs: Some(Self::jobs_or_default(matches).unwrap()),
            arch: Some(Self::arch_or_default(matches).unwrap()),
            out_dir: matches.get_one::<String>("make-out-dir").unwrap().clone(),
            kernel_dir: matches
                .get_one::<String>("make-kernel-dir")
                .unwrap()
                .clone(),
            extra_make_args: Vec::new(),
        };

        Ok(ret)
    }

    fn update_from_arg_matches(
        &mut self,
        matches: &ArgMatches,
    ) -> Result<(), clap::error::Error<clap::error::RichFormatter>> {
        self.path = matches.get_one::<String>("make-path").unwrap().clone();
        self.jobs = Some(Self::jobs_or_default(matches).unwrap());
        self.arch = Some(Self::arch_or_default(matches).unwrap());
        self.out_dir = matches.get_one::<String>("make-out-dir").unwrap().clone();
        self.kernel_dir = matches
            .get_one::<String>("make-kernel-dir")
            .unwrap()
            .clone();

        Ok(())
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn get_nprocs() -> Result<usize> {
    nix::unistd::sysconf(nix::unistd::SysconfVar::_NPROCESSORS_ONLN)
        .map(|v| v.unwrap() as usize)
        .context("Failed to get number of processors online")
}
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn get_nprocs() -> Result<usize> {
    let raw = unsafe {
        nix::errno::Errno::clear();
        nix::libc::sysconf(nix::libc::_SC_NPROCESSORS_ONLN)
    };
    if raw == -1 {
        Err(nix::errno::Errno::last().into())
    } else {
        Ok(raw as usize)
    }
}

fn get_arch() -> Result<Arch> {
    let uts = nix::sys::utsname::uname().context("Failed to get uname")?;

    uts.machine()
        .to_str()
        .context("Failed to get machine arch name")?
        .parse()
}
