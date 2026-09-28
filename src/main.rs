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

    std::fs::create_dir_all(db_path.parent().expect("Failed to get parent directory of database path"))?;
    let pool = connection::connect(&db_path).await?;
    schema::init_db(&pool).await?;

    match cli.command {
        Commands::Ingest { path } => {
            info!("Ingesting repo at {}", path.display());
            let path_str = path.to_str().expect("Invalid UTF-8 in path argument");
let (entities, relations) = ingest_repo(path_str, &pool).await?;
            info!("Found {} entities, {} relations", entities.len(), relations.len());
            insert_entities(&pool, &entities).await?;
            insert_relations(&pool, &relations).await?;
            info!("Ingested successfully.");
            println!("💡 Run `cortex index` to enable semantic search.");
        }
        Commands::Watch { path } => {
            info!("Starting watcher + API for {}", path.display());
            
            let api_pool = pool.clone();
            let api_handle = tokio::spawn(async move {
                if let Err(e) = cortex::api::start_server(api_pool).await {
                    eprintln!("API server error: {}", e);
                }
            });
            
            cortex::ingest::watch::start_watcher(path, pool).await?;
            api_handle.abort();
        }
        Commands::Serve => {
            info!("Starting Cortex API server");
            cortex::api::start_server(pool).await?;
        }
        Commands::Ask { query } => {
            let _ = cortex::core::events::log_event(&pool, "system", "query:asked", Some(&query)).await;
            
            // Push to working memory
            cortex::memory::stack::add_query(&query);
            
            let parsed = cortex::query::parser::parse_query(&query);
            let result = cortex::query::executor::execute_query(&pool, &parsed).await?;
            let formatted = cortex::query::formatter::format_result(&result);
            println!("{}", formatted);
            
            // If no results, suggest semantic search
            if result.entities.is_empty() {
                println!("\n💡 No keyword results. Try: cortex search \"{}\"", query);
            }
        }
        Commands::Search { query, limit } => {
            if !cortex::embed::client::check_ollama().await {
                eprintln!("🛑 Ollama is not running.");
                eprintln!("   Install: https://ollama.com");
                eprintln!("   Then run: ollama pull nomic-embed-text");
                eprintln!("   Then run: ollama serve");
                std::process::exit(1);
            }
            
            println!("🔍 Embedding query: {}", query);
            let embedding = cortex::embed::client::embed_text(&query).await?;
            
            println!("🔎 Searching...");
            let results = cortex::embed::search::semantic_search(&pool, &embedding, limit).await?;
            
            if results.is_empty() {
                println!("No results. Run `cortex index` first?");
            } else {
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Score", "Kind", "Name", "Source"]);
                
                for (entity_id, score) in results {
                    let row: Option<(String, String, String)> = sqlx::query_as(
                        "SELECT kind, name, source FROM entities WHERE id = ?"
                    )
                    .bind(&entity_id)
                    .fetch_optional(&pool)
                    .await?;
                    
                    if let Some((kind, name, source)) = row {
                        let _id_short = if entity_id.len() > 30 { format!("{}...", &entity_id[..30]) } else { entity_id };
                        table.add_row(vec![
                            format!("{:.3}", score),
                            kind,
                            name,
                            source,
                        ]);
                    }
                }
                println!("{}", table);
            }
        }
        Commands::Index => {
            if !cortex::embed::client::check_ollama().await {
                eprintln!("🛑 Ollama is not running.");
                eprintln!("   Install: https://ollama.com");
                eprintln!("   Then run: ollama pull nomic-embed-text");
                eprintln!("   Then run: ollama serve");
                std::process::exit(1);
            }
            
            cortex::embed::indexer::index_all_entities(&pool).await?;
        }
        Commands::Timeline { when } => {
            let range = cortex::temporal::parser::parse_time_expression(&when)
                .unwrap_or_else(|| {
                    println!("⚠️ Could not parse '{}'. Using 'today'.", when);
                    cortex::temporal::parser::parse_time_expression("today").expect("Failed to parse fallback time expression 'today'")
                });
            
            let sessions = cortex::temporal::reconstructor::get_timeline(&pool, &range).await?;
            let formatted = cortex::temporal::formatter::format_timeline(&sessions);
            println!("{}", formatted);
        }
        Commands::Recall { when } => {
            let range = cortex::temporal::parser::parse_time_expression(&when)
                .unwrap_or_else(|| {
                    println!("⚠️ Could not parse '{}'. Using '2 hours ago'.", when);
                    cortex::temporal::parser::parse_time_expression("2 hours ago").expect("Failed to parse fallback time expression '2 hours ago'")
                });
            
            let point = if range.is_point {
                range.start + chrono::Duration::minutes(30)
            } else {
                range.start
            };
            
            let state = cortex::temporal::reconstructor::reconstruct_state(&pool, point).await?;
            let formatted = cortex::temporal::formatter::format_mental_state(&state);
            println!("{}", formatted);
        }
        Commands::Focus => {
            println!("{}", cortex::memory::formatter::format_focus());
        }
        Commands::Stack => {
            println!("{}", cortex::memory::formatter::format_stack());
        }
        Commands::Back => {
            match cortex::memory::stack::back() {
                Some(prev) => {
                    println!("{}", cortex::memory::formatter::format_back_result(&prev));
                }
                None => {
                    println!("No previous context in stack. You're at the bottom.");
                }
            }
        }
        Commands::Where => {
            println!("{}", cortex::memory::formatter::format_where_was_i());
        }
        Commands::Debt => {
            let debts = cortex::debt::detector::detect_debt(&pool).await?;
            println!("{}", cortex::debt::formatter::format_debt_report(&debts));
        }
        Commands::Weekly => {
            let debts = cortex::debt::detector::detect_debt(&pool).await?;
            println!("{}", cortex::debt::formatter::format_weekly_report(&debts));
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
            let event_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
                .fetch_one(&pool)
                .await?;
            let embed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM embeddings")
                .fetch_one(&pool)
                .await?;
            println!("Total entities: {}", count);
            println!("Live filesystem entities: {}", live_count);
            println!("Events recorded: {}", event_count);
            println!("Embeddings indexed: {}", embed_count);
        }
        Commands::Version => {
            println!("╔══════════════════════════════════════════════════════╗");
            println!("║              🧠 CORTEX — VERSION INFO               ║");
            println!("╚══════════════════════════════════════════════════════╝");
            println!();
            println!("  Version:    {}", env!("CARGO_PKG_VERSION"));
            println!("  Build:      Rust {} edition", "2021");
            println!("  License:    MIT");
            println!("  Repository: https://github.com/pranavganesh1/Cortex");
            println!();
            println!("  Data dir:   {}", db_path.parent().unwrap_or(&db_path).display());
            println!("  Database:   {}", db_path.display());
            println!("  OS:         {} {}", std::env::consts::OS, std::env::consts::ARCH);
            println!();
            
            // Show quick stats if DB exists
            if db_path.exists() {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);
                let embed_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM embeddings")
                    .fetch_one(&pool)
                    .await
                    .unwrap_or(0);
                println!("  Entities:   {}", count);
                println!("  Embeddings: {}", embed_count);
            } else {
                println!("  Database:   not initialized (run `cortex ingest` first)");
            }
        }
    }

    Ok(())
}
