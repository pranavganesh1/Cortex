use sqlx::SqlitePool;
use anyhow::Result;
use crate::embed::client::embed_text;

pub async fn index_entity(pool: &SqlitePool, entity_id: &str, text: &str) -> Result<()> {
    if text.trim().is_empty() {
        return Ok(());
    }
    
    let embedding = embed_text(text).await?;
    let vector_json = serde_json::to_string(&embedding)?;
    
    sqlx::query(
        "INSERT OR REPLACE INTO embeddings (entity_id, vector, model, created_at) VALUES (?, ?, ?, ?)"
    )
    .bind(entity_id)
    .bind(&vector_json)
    .bind("nomic-embed-text")
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn index_all_entities(pool: &SqlitePool) -> Result<usize> {
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, content FROM entities WHERE kind IN ('File', 'Function', 'Struct', 'Class', 'Trait', 'Enum', 'Decision')"
    )
    .fetch_all(pool)
    .await?;
    
    let total = rows.len();
    let mut count = 0;
    println!("🔮 Indexing {} entities with Ollama (nomic-embed-text)...", total);
    
    for (id, name, content) in rows {
        let text = if let Some(c) = content {
            format!("{} {}", name, c.lines().take(20).collect::<Vec<_>>().join("\n"))
        } else {
            name
        };
        
        match index_entity(pool, &id, &text).await {
            Ok(()) => {
                count += 1;
                if count % 10 == 0 {
                    println!("  ...{}/{} done", count, total);
                }
            }
            Err(e) => {
                eprintln!("  ⚠️  {}: {}", id, e);
            }
        }
    }
    
    println!("✅ Indexed {} entities", count);
    Ok(count)
}

pub async fn index_file(pool: &SqlitePool, entity_id: &str, name: &str, content: &str) {
    let text = format!("{} {}", name, content.lines().take(30).collect::<Vec<_>>().join("\n"));
    if let Err(e) = index_entity(pool, entity_id, &text).await {
        tracing::debug!("Embedding failed for {}: {}", entity_id, e);
    }
}
