use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cortex")]
#[command(about = "Universal AI memory layer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Ingest Git history (one-time)
    Ingest {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Watch filesystem for live changes
    Watch {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Query the knowledge graph
    Ask {
        query: String,
    },
    /// Show stats
    Status,
}
