use clap::{Arg, Command};
use anyhow::Result;

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

async fn async_main(config: config::Config) -> Result<()> {
    let make_common_args = make::common_args(&config);

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
                .args(&make_common_args)
                .groups(make::common_groups()),
        )
        .subcommand(
            Command::new("config")
                .args(&make_common_args)
                .groups(make::common_groups()),
        );

    let matches = app.get_matches();

    println!("{:?}", matches);
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
