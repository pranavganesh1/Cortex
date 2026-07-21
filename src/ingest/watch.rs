use notify::{Event, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use anyhow::Result;
use sqlx::SqlitePool;
use crate::core::models::{Entity, EntityKind};
use crate::db::{upsert_entity, delete_entities_for_file, insert_entities, insert_relations};
use crate::ingest::parser::parse_file;
use chrono::Utc;

pub async fn start_watcher(path: PathBuf, pool: SqlitePool) -> Result<()> {
    let (notify_tx, notify_rx) = channel::<notify::Event>();

    let mut watcher = notify::recommended_watcher(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = notify_tx.send(event);
            }
        }
    )?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    println!("👁️  Watching {} for changes... (Ctrl+C to stop)", path.display());

    let handle = tokio::runtime::Handle::current();
    let root = path.clone();

    tokio::task::spawn_blocking(move || {
        let mut pending = std::collections::HashSet::<PathBuf>::new();
        let mut last_flush = std::time::Instant::now();

        loop {
            match notify_rx.recv_timeout(Duration::from_millis(300)) {
                Ok(event) => {
                    if !event.kind.is_modify() && !event.kind.is_create() {
                        continue;
                    }
                    for p in event.paths {
                        if !should_ignore(&p) && is_code_file(&p) {
                            pending.insert(p);
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }

            if !pending.is_empty() && last_flush.elapsed() > Duration::from_millis(800) {
                let batch: Vec<_> = pending.drain().collect();
                handle.block_on(async {
                    for file_path in batch {
                        if let Err(e) = process_file(&file_path, &root, &pool).await {
                            eprintln!("⚠️  {}", e);
                        }
                    }
                });
                last_flush = std::time::Instant::now();
            }
        }
    }).await?;

    Ok(())
}

fn should_ignore(path: &Path) -> bool {
    path.components().any(|c| {
        let name = c.as_os_str().to_string_lossy();
        name == ".git"
            || name == "target"
            || name == "node_modules"
            || name == ".cortex"
            || name == "dist"
            || name == "build"
    })
}

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs") | Some("py") | Some("js") | Some("ts") | Some("jsx") | Some("tsx")
        | Some("go") | Some("java") | Some("c") | Some("cpp") | Some("h") | Some("hpp")
        | Some("md") | Some("txt") | Some("yaml") | Some("yml") | Some("json")
        | Some("toml")
    )
}

async fn process_file(path: &Path, root: &Path, pool: &SqlitePool) -> Result<()> {
    let content = match tokio::fs::read_to_string(path).await {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();
    let file_id = format!("file:{}", relative);
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

    // Track as active file for context injection
    crate::core::state::set_active_file(&relative);

    // Push to working memory stack
    crate::memory::stack::touch_file(&relative);

    // 1. Upsert the file entity
    let entity = Entity {
        id: file_id.clone(),
        kind: EntityKind::File,
        name,
        content: Some(content.clone()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source: "filesystem".to_string(),
        parent_id: None,
    };
    upsert_entity(pool, &entity).await?;
    // Log file save event
    let _ = crate::core::events::log_event(pool, &file_id, "file:save", Some(&relative)).await;
    println!("✓ {}", relative);

    // 2. If it's a parseable code file, delete old code entities and re-parse
    if is_parseable(path) {
        delete_entities_for_file(pool, &file_id).await?;

        if let Some((entities, relations)) = parse_file(path, &content) {
            insert_entities(pool, &entities).await?;
            insert_relations(pool, &relations).await?;
            println!("  └─ {} entities, {} relations", entities.len(), relations.len());
        }
    }

    // Extract decisions from code comments
    let comment_decisions = crate::extract::decisions::extract_from_comments(&content, &relative);
    crate::extract::decisions::store_decisions(pool, &comment_decisions, Some(file_id.clone())).await?;

    // Auto-index for semantic search (fire and forget)
    let pool_clone = pool.clone();
    let file_id_clone = file_id.clone();
    let name_clone = entity.name.clone();
    let content_clone = content.clone();
    tokio::spawn(async move {
        let text = format!("{} {}", name_clone, content_clone.lines().take(30).collect::<Vec<_>>().join("\n"));
        let _ = crate::embed::indexer::index_entity(&pool_clone, &file_id_clone, &text).await;
    });

    Ok(())
}

fn is_parseable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs") | Some("py") | Some("js") | Some("ts") | Some("jsx") | Some("tsx") | Some("go")
    )
}
