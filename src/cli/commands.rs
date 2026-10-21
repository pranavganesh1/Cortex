use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, PartialEq)]
#[command(name = "cortex")]
#[command(about = "Universal AI memory layer")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Recall what you were doing at a specific time
    Recall {
        when: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show current focus / working memory
    Focus,
    /// Show working memory stack
    Stack {
        /// Clear the working memory stack
        #[arg(long)]
        clear: bool,
    },
    /// Pop back to previous context
    Back,
    /// "Where was I?" — show recent context switches
    Where,
    /// Show cognitive debt dashboard
    Debt {
        /// Minimum severity level (low, medium, high, critical)
        #[arg(short = 's', long)]
        min_severity: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show weekly cognitive report
    Weekly,
    /// Show stats
    Status,
    /// Show version and build information
    Version,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_focus() {
        let args = ["cortex", "focus"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse focus command");
        assert_eq!(cli.command, Commands::Focus);
    }

    #[test]
    fn test_parse_cli_ask() {
        let args = ["cortex", "ask", "why sqlite?"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse ask command");
        assert_eq!(
            cli.command,
            Commands::Ask {
                query: "why sqlite?".to_string()
            }
        );
    }

    #[test]
    fn test_parse_cli_search_defaults_and_flags() {
        let default_args = ["cortex", "search", "authentication"];
        let cli = Cli::try_parse_from(default_args).expect("Failed to parse search command");
        assert_eq!(
            cli.command,
            Commands::Search {
                query: "authentication".to_string(),
                limit: 10
            }
        );

        let flag_args = ["cortex", "search", "authentication", "-l", "5"];
        let cli_flag = Cli::try_parse_from(flag_args).expect("Failed to parse search with flag");
        assert_eq!(
            cli_flag.command,
            Commands::Search {
                query: "authentication".to_string(),
                limit: 5
            }
        );
    }

    #[test]
    fn test_parse_cli_timeline_default() {
        let args = ["cortex", "timeline"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse timeline command");
        assert_eq!(
            cli.command,
            Commands::Timeline {
                when: "today".to_string(),
                json: false,
            }
        );
    }

    #[test]
    fn test_parse_cli_timeline_flags() {
        let args = ["cortex", "timeline", "this week", "--json"];
        let cli = Cli::try_parse_from(args).expect("Failed to parse timeline with json");
        assert_eq!(
            cli.command,
            Commands::Timeline {
                when: "this week".to_string(),
                json: true,
            }
        );
    }

    #[test]
    fn test_parse_cli_debt_and_status() {
        let debt_cli = Cli::try_parse_from(["cortex", "debt"]).unwrap();
        assert_eq!(
            debt_cli.command,
            Commands::Debt {
                min_severity: None,
                json: false,
            }
        );

        let debt_filter_cli = Cli::try_parse_from(["cortex", "debt", "-s", "high", "--json"]).unwrap();
        assert_eq!(
            debt_filter_cli.command,
            Commands::Debt {
                min_severity: Some("high".to_string()),
                json: true,
            }
        );

        let status_cli = Cli::try_parse_from(["cortex", "status"]).unwrap();
        assert_eq!(status_cli.command, Commands::Status);
    }

    #[test]
    fn test_parse_cli_stack_flags() {
        let stack_cli = Cli::try_parse_from(["cortex", "stack"]).unwrap();
        assert_eq!(stack_cli.command, Commands::Stack { clear: false });

        let clear_cli = Cli::try_parse_from(["cortex", "stack", "--clear"]).unwrap();
        assert_eq!(clear_cli.command, Commands::Stack { clear: true });
    }
}
