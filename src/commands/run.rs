use crate::boot::QemuCmd;
use crate::config::Config;
use crate::Result;
use clap::FromArgMatches;
use tracing::*;

pub fn command(config: &Config) -> clap::Command {
    let cmd = clap::Command::new("run")
        .about("Boot the kernel")
        .arg(
            clap::Arg::new("make-args")
                .long("args")
                .action(clap::ArgAction::Append)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("no-build")
                .long("no-build")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("parse-only")
                .long("parse-only")
                .action(clap::ArgAction::SetTrue)
                .hide(true),
        )
        .arg(
            clap::Arg::new("test")
                .required(true)
                .value_parser(clap::value_parser!(String))
                .value_hint(clap::ValueHint::ExecutablePath)
                .index(1),
        );
    config.qemu.augument_args(cmd)
}

#[instrument(name = "run", level = "debug", skip(config, matches))]
pub fn run(config: &mut Config, matches: &clap::ArgMatches) -> Result {
    config.qemu.update_from_arg_matches(matches)?;

    let test = matches.get_one::<String>("test").unwrap();
    let deps = ktest_runner::Deps::get(config, test)?;
    deps.merge_into_config(config);

    if matches.get_flag("parse-only") {
        trace!("config: {config:#?}");
        deps.print_shell();
        return Ok(());
    }

    if !matches.get_flag("no-build") {
        let args = matches.get_many::<String>("make-args").unwrap_or_default();
        crate::build::build(config, args)?;
    }

    QemuCmd::new(config)?.run()?;

    Ok(())
}
