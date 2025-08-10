use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cc-monitor")]
#[command(about = "Monitor Claude Code usage with an interactive dashboard")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show current usage status
    Status {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,
        
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    
    /// Launch interactive dashboard
    Dashboard,
    
    /// Show daily usage report
    Daily {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
        
        /// Number of days to show
        #[arg(short, long, default_value = "7")]
        days: usize,
    },
    
    /// Show monthly usage report
    Monthly {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    
    /// Show session-based usage report
    Sessions {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
        
        /// Number of sessions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}