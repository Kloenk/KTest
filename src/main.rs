use anyhow::{bail, Context, Result};
use clap::{Arg, Command, FromArgMatches};

mod commands;
mod config;
mod make;

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
    let mut config = config::Config::new().expect("Failed to read config");
    let rt = config.init().expect("Failed to initialize async runtime");

    if let Err(e) = rt.block_on(async move { async_main(config).await }) {
        eprintln!("Failed to execute ktest:");
        eprint!("{e}");
    }
}

async fn async_main(mut config: config::Config) -> Result<()> {
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
        .subcommand(
            commands::make::command(&config)
        )
        .subcommand(
            commands::config::command(&config)
        )
        .subcommand(
            commands::oldconfig::command(&config)
        )
        /*.subcommand(
            Command::new("config")
                .args(&make_common_args)
                .groups(make::common_groups()),
        )*/;

    let matches = app.get_matches();

    config.make.update_from_arg_matches(&matches)?;

    println!("{:?}", matches.get_one::<String>("make-path"));
    println!("{config:?}");
    println!("{}", config.make.arch.as_ref().unwrap());

    match matches.subcommand().context("No subcomand provided")? {
        ("make", matches) => make::make(&config, &matches).await?,

        ("config", matches) => commands::config::run(&config, &matches).await?,
        ("oldconfig", matches) => commands::oldconfig::run(&config, &matches).await?,

        _ => bail!("Unknown subcommand"),
    };
    Ok(())
}

/*
#[tokio::main]
async fn main() -> std::io::Result<()> {
    //let cli = Cli::parse();
    let app = clap::command!()
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count),
        )
        .group(clap::ArgGroup::new("make-args").multiple(true))
        .subcommand_required(true)
        .subcommand(
            Command::new("make")
                .args(make::common_args())
                .groups(make::common_groups()),
        )
        .subcommand(
            Command::new("config")
                .args(make::common_args())
                .groups(make::common_groups()),
        );

    let matches = app.get_matches();

    println!("{:?}", matches);

    Ok(())
}*/
