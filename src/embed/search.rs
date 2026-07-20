use serde_json;
use sqlx::SqlitePool;
use anyhow::Result;

pub async fn semantic_search(pool: &SqlitePool, query_embedding: &[f32], limit: usize) -> Result<Vec<(String, f32)>> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT entity_id, vector FROM embeddings")
        .fetch_all(pool)
        .await?;
    
    let mut scored: Vec<(String, f32)> = rows.into_iter()
        .filter_map(|(entity_id, vector_json)| {
            let vector: Vec<f32> = serde_json::from_str(&vector_json).ok()?;
            let score = cosine_similarity(query_embedding, &vector);
            Some((entity_id, score))
        })
        .collect();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
