use clap::FromArgMatches;
use ktest_core::config::Config;
use ktest_core::Result;
use tracing::*;

pub fn command(config: &Config) -> clap::Command {
    let cmd = clap::Command::new("deps")
        .about("Show requirements for a given test")
        .arg(
            clap::Arg::new("test")
                .required(true)
                .value_parser(clap::value_parser!(String))
                .value_hint(clap::ValueHint::ExecutablePath)
                .index(1),
        );

    config.ktest.augument_args(cmd)
}

#[instrument(name = "run", level = "debug", skip(config, matches))]
pub fn run(config: &mut Config, matches: &clap::ArgMatches) -> Result {
    config.ktest.update_from_arg_matches(matches)?;
    let test = matches.get_one::<String>("test").unwrap();

    let deps = ktest_runner::Deps::get(config, test)?;

    deps.print_shell();

    Ok(())
}
