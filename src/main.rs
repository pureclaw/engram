mod cli;
mod db;
mod embed;
mod index;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => index::init()?,
        Commands::Add { paths, recursive } => index::add(&paths, recursive)?,
        Commands::Search { query, limit, show_path } => index::search(&query, limit, show_path)?,
        Commands::Remove { paths } => index::remove(&paths)?,
        Commands::Rebuild => index::rebuild()?,
        Commands::Status => index::status()?,
    }
    Ok(())
}
