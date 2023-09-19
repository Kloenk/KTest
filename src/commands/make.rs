use crate::config::Config;
use clap::{Arg, Command};

pub fn command(_config: &Config) -> Command {
    Command::new("make").arg(
        Arg::new("make-args")
            .action(clap::ArgAction::Append)
            .value_parser(clap::value_parser!(String))
            .trailing_var_arg(true),
    )
}
