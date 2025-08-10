use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cc-monitor")]
#[command(about = "Monitor Claude Code usage")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive dashboard
    Dashboard,
    
    /// Show compact statusline (for use with Claude hooks)
    Statusline {
        /// Read JSON input from stdin (for hook integration)
        #[arg(long)]
        stdin: bool,
    },
}