use crate::boot::QemuCmd;
use crate::config::Config;
use crate::Result;
use clap::FromArgMatches;
use tracing::*;

pub fn command(config: &Config) -> clap::Command {
    let cmd = clap::Command::new("boot")
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
        );
    config.qemu.augument_args(cmd)
}

#[instrument(name = "boot", level = "debug", skip(config, matches))]
pub fn run(config: &mut Config, matches: &clap::ArgMatches) -> Result {
    config.qemu.update_from_arg_matches(matches)?;

    if !matches.get_flag("no-build") {
        let args = matches.get_many::<String>("make-args").unwrap_or_default();
        crate::build::build(config, args)?;
    }

    trace!("using qemu: {:?}", config.qemu_path());

    //crate::boot::boot(config, args)?;
    QemuCmd::new(config)?.run()?;

    Ok(())
}
