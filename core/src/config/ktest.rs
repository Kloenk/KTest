use crate::{Context, Result};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Error, FromArgMatches};
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::*;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct KTest {
    pub lib_path: Option<String>,
}

impl KTest {
    pub fn augument_args(&self, cmd: clap::Command) -> clap::Command {
        cmd.arg(
            Arg::new("ktest-lib-path")
                .long("ktest-lib")
                .action(ArgAction::Set)
                .value_name("PATH")
                .value_hint(clap::ValueHint::DirPath)
                .value_parser(value_parser!(String))
                .hide(true),
        )
    }

    #[cfg(debug_assertions)]
    pub fn lib_path(&self) -> Result<PathBuf> {
        if let Some(path) = &self.lib_path {
            let path = Path::new(path).join("testlib.sh");
            if !path.exists() {
                return Err("configured KTEST_TEST_LIB does not exist".into());
            }
            Ok(path)
        } else {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"));
            let path = path.parent().unwrap().join("lib").join("testlib.sh");
            eprintln!(
                "Using relative resolve to find testlib.sh, this is only supported in debug mode",
            );
            eprintln!("Found {path}", path = path.display().to_string(),);
            Ok(path)
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn lib_path(&self) -> Result<PathBuf> {
        let path = self
            .lib_path
            .as_ref()
            .context("KTEST_TEST_LIB not configured")?;
        let path = Path::new(path).join("testlib.sh");
        if !path.exists() {
            return Err("configured KTEST_TEST_LIB does not exist".into());
        }
        Ok(path)
    }
}

impl FromArgMatches for KTest {
    fn from_arg_matches(matches: &clap::ArgMatches) -> std::result::Result<Self, Error> {
        unimplemented!()
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> std::result::Result<(), Error> {
        self.lib_path = matches
            .get_one::<String>("ktest-lib-path")
            .map(|s| s.to_string());

        Ok(())
    }
}
