mod cli;
mod commands;
mod data_loader;
mod models;
mod tui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Commands};
use commands::show_statusline;
use data_loader::DataLoader;
use tui::{App, run_dashboard};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("warn"))
        )
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Dashboard) | None => {
            // Dashboard is the default command
            let loader = DataLoader::new()?;
            let stats = loader.load_all_usage()?;
            let app = App::new(stats);
            run_dashboard(app)?;
        }
        Some(Commands::Statusline { stdin }) => {
            show_statusline(stdin)?;
        }
    }
    
    Ok(())
}