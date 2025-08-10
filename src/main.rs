mod cli;
mod commands;
mod data_loader;
mod models;
mod tui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Commands};
use commands::{show_daily, show_monthly, show_sessions, show_status};
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
        Commands::Status { detailed, json } => {
            show_status(detailed, json)?;
        }
        Commands::Dashboard => {
            let loader = DataLoader::new()?;
            let stats = loader.load_all_usage()?;
            let app = App::new(stats);
            run_dashboard(app)?;
        }
        Commands::Daily { json, days } => {
            show_daily(json, days)?;
        }
        Commands::Monthly { json } => {
            show_monthly(json)?;
        }
        Commands::Sessions { json, limit } => {
            show_sessions(json, limit)?;
        }
    }
    
    Ok(())
}
