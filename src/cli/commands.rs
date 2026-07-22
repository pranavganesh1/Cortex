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
    /// Watch filesystem for live changes + API server
    Watch {
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Start API server only
    Serve,
    /// Query the knowledge graph
    Ask {
        query: String,
    },
    /// Semantic search with vector embeddings
    Search {
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Index all entities for semantic search
    Index,
    /// Show work timeline
    Timeline {
        #[arg(default_value = "today")]
        when: String,
    },
    /// Recall what you were doing at a specific time
    Recall {
        when: String,
    },
    /// Show current focus / working memory
    Focus,
    /// Show working memory stack
    Stack,
    /// Pop back to previous context
    Back,
    /// "Where was I?" — show recent context switches
    Where,
    /// Show cognitive debt dashboard
    Debt,
    /// Show weekly cognitive report
    Weekly,
    /// Show stats
    Status,
}
