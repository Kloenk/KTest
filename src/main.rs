use clap::{Arg, FromArgMatches};
use tracing::trace;

mod build;
mod commands;
mod config;
mod err;
mod kconfig;
mod make;

pub use err::{Context, Error, ErrorKind, Result};

/*#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[derive(clap::Subcommand)]
enum Commands {
    Make {
        #[command(flattern)]
        commonargs: make::CommandMakeArgs,
    }
}*/

pub fn main() {
    let config = config::Config::new().expect("Failed to read config");
    config.init().expect("Failed to initialize async runtime");

    if let Err(e) = run_main(config) {
        trace!("Failed to run ktest: {e:?}");
        e.exit();
    }
}

fn run_main(mut config: config::Config) -> Result {
    let app = clap::command!()
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        )
        .subcommand_required(true)
        .subcommand(commands::make::command(&config))
        .subcommand(commands::config::command(&config))
        .subcommand(commands::oldconfig::command(&config))
        .subcommand(commands::build::command(&config));
    let app = config.make.augument_args(app);

    let matches = app.get_matches();

    config
        .make
        .update_from_arg_matches(&matches)
        .context("Failed to parse matches")?;
    trace!("Loaded config: {config:?}");

    match matches.subcommand().context("No subcomand provided")? {
        ("make", matches) => commands::make::run(&config, &matches)?,
        //make::make(&config, &matches).await?,
        ("config", matches) => commands::config::run(&config, &matches)?,
        ("oldconfig", matches) => commands::oldconfig::run(&config, &matches)?,
        ("build", matches) => commands::build::run(&config, &matches)?,

        _ => return Err(Error::new("Unknown subcommand")),
    };
    Ok(())
}
