use clap::Parser;
use dotenvy::dotenv;

mod cli;
mod domain;
mod repository;
mod utils;

fn main() {
    dotenv().ok();

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Domain(cmd) => cmd.run(),
        cli::Commands::Repository(cmd) => cmd.run(),
    }
}
