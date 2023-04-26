use anyhow::Ok;
use clap::Parser;

mod cli;
use cli::{Cli, Commands};

mod utils;

fn process_cli(cli: &Cli) -> anyhow::Result<String> {
    match cli.command {
        Commands::Md5 { addr, size } => {
            let md5 = utils::get_md5(addr, size)?;
            println!("md5: {md5}");
        },
        Commands::Write { addr, size, value } => {
            utils::write(addr, size, value)?;
            println!("write success");
        },
        Commands::Clear { addr, size, value } => {
            utils::clear(addr, size, value)?;
            println!("clear success");
        },
        Commands::Read { addr, size } => {
            utils::read(addr, size)?;
        },
        Commands::MD { addr, unit, count } => {
            utils::mem_dump(addr, unit, count)?;
        },
    }

    Ok(String::new())
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = process_cli(&cli) {
        println!("failed: {error}");
    }
}
