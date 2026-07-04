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
    /// Ingest a Git repository
    Ingest {
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
