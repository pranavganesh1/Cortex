use sqlx::SqlitePool;
use anyhow::Result;

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct EntityRow {
    id: String,
    kind: String,
    name: String,
    content: Option<String>,
    created_at: String,
    updated_at: String,
    source: String,
    parent_id: Option<String>,
}

pub async fn assemble_context(pool: &SqlitePool, file_path: &str, user_query: &str) -> Result<String> {
    let file_id = format!("file:{}", file_path);
    
    // 1. Find file entity (exact ID first, then by name fallback)
    let mut file: Option<EntityRow> = sqlx::query_as("SELECT * FROM entities WHERE id = ?")
        .bind(&file_id)
        .fetch_optional(pool)
        .await?;
    
    if file.is_none() {
        let filename = std::path::Path::new(file_path).file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        file = sqlx::query_as("SELECT * FROM entities WHERE name = ? AND kind = 'File' LIMIT 1")
            .bind(&filename)
            .fetch_optional(pool)
            .await?;
    }
    
    let effective_id = file.as_ref().map(|f| f.id.clone()).unwrap_or_else(|| file_id.clone());
    
    // 2. Get symbols (functions, structs, etc.) contained in this file
    let children: Vec<EntityRow> = sqlx::query_as(
        "SELECT e.* FROM entities e 
         JOIN relations r ON e.id = r.to_id 
         WHERE r.from_id = ? AND r.kind = 'Contains'"
    )
    .bind(&effective_id)
    .fetch_all(pool)
    .await?;
    
    // 3. Get recent commits that touched this file
    let commits: Vec<EntityRow> = sqlx::query_as(
        "SELECT e.* FROM entities e 
         JOIN relations r ON e.id = r.from_id 
         WHERE r.to_id = ? AND r.kind = 'Modified'
         ORDER BY e.created_at DESC LIMIT 5"
    )
    .bind(&effective_id)
    .fetch_all(pool)
    .await?;
    
    // 4. Get related files (modified in same commits)
    let related_files: Vec<EntityRow> = sqlx::query_as(
        "SELECT DISTINCT e.* FROM entities e
         JOIN relations r1 ON e.id = r1.to_id
         JOIN relations r2 ON r1.from_id = r2.from_id
         WHERE r2.to_id = ? AND r1.to_id != ? AND e.kind = 'File'
         LIMIT 10"
    )
    .bind(&effective_id)
    .bind(&effective_id)
    .fetch_all(pool)
    .await?;
    
    // 5. Assemble the context block
    let mut output = String::new();
    output.push_str("=== CORTEX PROJECT CONTEXT ===\n\n");
    
    if let Some(f) = file {
        output.push_str(&format!("Current file: {}\n", f.name));
        if let Some(content) = &f.content {
            let lines: Vec<&str> = content.lines().collect();
            let preview = if lines.len() > 30 {
                lines[..30].join("\n")
            } else {
                content.clone()
            };
            output.push_str(&format!("\nFile preview (first 30 lines):\n```\n{}\n```\n", preview));
        }
    } else {
        output.push_str(&format!("Current file: {} (not yet indexed by Cortex)\n", file_path));
    }
    
    if !children.is_empty() {
        output.push_str("\nSymbols in this file:\n");
        for child in &children {
            let icon = match child.kind.as_str() {
                "Function" => "f",
                "Struct" => "S",
                "Class" => "C",
                "Enum" => "E",
                "Trait" => "T",
                _ => "•",
            };
            output.push_str(&format!("  {} {} ({})\n", icon, child.name, child.kind));
        }
    }
    
    if !commits.is_empty() {
        output.push_str("\nRecent changes:\n");
        for commit in &commits {
            let msg = commit.content.as_deref().unwrap_or("").lines().next().unwrap_or("no message");
            let short_name = if commit.name.len() > 50 { &commit.name[..50] } else { &commit.name };
            output.push_str(&format!("  • [{}] {}\n", short_name, msg));
        }
    }
    
    if !related_files.is_empty() {
        output.push_str("\nRelated files (changed together):\n");
        for rf in &related_files {
            output.push_str(&format!("  • {}\n", rf.name));
        }
    }
    
    // 6. Project stats
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
        .fetch_one(pool)
        .await?;
    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities WHERE kind = 'File'")
        .fetch_one(pool)
        .await?;
    
    output.push_str(&format!("\nProject stats: {} entities, {} files indexed\n", total, total_files));
    output.push_str("\n=== END CONTEXT ===\n\n");
    output.push_str(&format!("User question: {}\n", user_query));
    output.push_str("\n---\nPlease answer using the context above.\n");
    
    Ok(output)
}
