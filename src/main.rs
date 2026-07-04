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
            let (entities, relations) = ingest_repo(path.to_str().unwrap())?;
            info!("Found {} entities, {} relations", entities.len(), relations.len());
            insert_entities(&pool, &entities).await?;
            insert_relations(&pool, &relations).await?;
            info!("Ingested successfully.");
        }
        Commands::Ask { query } => {
            println!("Query: {} (not implemented yet — Day 3)", query);
        }
        Commands::Status => {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
                .fetch_one(&pool)
                .await?;
            println!("Total entities in memory: {}", count);
        }
    }

    Ok(())
}
