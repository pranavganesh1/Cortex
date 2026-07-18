use clap::Parser;
use cortex::cli::commands::{Cli, Commands};
use cortex::db::{connection, schema, insert_entities, insert_relations};
use cortex::ingest::git::ingest_repo;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let db_path = dirs::home_dir()
        .unwrap_or(PathBuf::from("."))
        .join(".cortex")
        .join("cortex.db");

    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let pool = connection::connect(&db_path).await?;
    schema::init_db(&pool).await?;

    match cli.command {
        Commands::Ingest { path } => {
            info!("Ingesting repo at {}", path.display());
            let (entities, relations) = ingest_repo(path.to_str().unwrap(), &pool).await?;
            info!("Found {} entities, {} relations", entities.len(), relations.len());
            insert_entities(&pool, &entities).await?;
            insert_relations(&pool, &relations).await?;
            info!("Ingested successfully.");
        }
        Commands::Watch { path } => {
            info!("Starting watcher + API for {}", path.display());
            
            // Start API server in background
            let api_pool = pool.clone();
            let api_handle = tokio::spawn(async move {
                if let Err(e) = cortex::api::start_server(api_pool).await {
                    eprintln!("API server error: {}", e);
                }
            });
            
            // Start file watcher (blocks until Ctrl+C)
            cortex::ingest::watch::start_watcher(path, pool).await?;
            
            api_handle.abort();
        }
        Commands::Serve => {
            info!("Starting Cortex API server");
            cortex::api::start_server(pool).await?;
        }
        Commands::Ask { query } => {
            let parsed = cortex::query::parser::parse_query(&query);
            let result = cortex::query::executor::execute_query(&pool, &parsed).await?;
            let formatted = cortex::query::formatter::format_result(&result);
            println!("{}", formatted);
        }
        Commands::Status => {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
                .fetch_one(&pool)
                .await?;
            let live_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM entities WHERE source = 'filesystem'"
            )
            .fetch_one(&pool)
            .await?;
            println!("Total entities: {}", count);
            println!("Live filesystem entities: {}", live_count);
        }
    }

    Ok(())
}
