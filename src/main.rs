use clap::{Arg, FromArgMatches};
use tracing::trace;

mod commands;
mod config;
mod err;
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
    let rt = config.init().expect("Failed to initialize async runtime");

    if let Err(e) = rt.block_on(async move { async_main(config).await }) {
        trace!("Failed to run ktest: {e:?}");
        e.exit();
    }
}

async fn async_main(mut config: config::Config) -> Result {
    let app = clap::command!()
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        )
        .args(config.make.args())
        .group(clap::ArgGroup::new("make-args").multiple(true))
        .subcommand_required(true)
        .subcommand(commands::make::command(&config))
        .subcommand(commands::config::command(&config))
        .subcommand(commands::oldconfig::command(&config));

    let matches = app.get_matches();

    config
        .make
        .update_from_arg_matches(&matches)
        .context("Failed to parse matches")?;

    match matches.subcommand().context("No subcomand provided")? {
        ("make", matches) => commands::make::run(&config, &matches).await?,
        //make::make(&config, &matches).await?,
        ("config", matches) => commands::config::run(&config, &matches).await?,
        ("oldconfig", matches) => commands::oldconfig::run(&config, &matches).await?,

        _ => return Err(Error::new("Unknown subcommand")),
    };
    Ok(())
}
