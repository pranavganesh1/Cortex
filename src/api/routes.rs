use axum::{extract::{Query, State}, response::Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ContextParams {
    file_path: String,
    query: String,
}

#[derive(Deserialize)]
pub struct ActiveContextParams {
    query: Option<String>,
}

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn status(State(state): State<super::AppState>) -> Result<Json<serde_json::Value>, String> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    
    let files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM entities WHERE kind = 'File'")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(Json(json!({
        "status": "ok",
        "total_entities": count,
        "total_files": files
    })))
}

pub async fn context(
    State(state): State<super::AppState>,
    Query(params): Query<ContextParams>,
) -> Result<String, String> {
    let ctx = super::context::assemble_context(&state.pool, &params.file_path, &params.query)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ctx)
}

pub async fn active_context(
    State(state): State<super::AppState>,
    Query(params): Query<ActiveContextParams>,
) -> Result<String, String> {
    let active = crate::core::state::get_active_file();
    
    let file_path = match active {
        Some(f) => f,
        None => {
            return Ok("=== CORTEX ===\nNo active file tracked yet. Open and save a file in your editor first.\n==============".to_string());
        }
    };
    
    let query = params.query.unwrap_or_else(|| "Explain this code".to_string());
    let ctx = super::context::assemble_context(&state.pool, &file_path, &query)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ctx)
}

pub async fn focus(State(_state): State<super::AppState>) -> Result<String, String> {
    let output = crate::memory::formatter::format_focus();
    Ok(output)
}

#[derive(Deserialize)]
pub struct ListParams {
    limit: Option<usize>,
    kind: Option<String>,
}

pub async fn entities(
    State(state): State<super::AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, String> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows: Vec<(String, String, String, String)> = if let Some(kind) = params.kind {
        sqlx::query_as("SELECT id, kind, name, source FROM entities WHERE kind = ? ORDER BY created_at DESC LIMIT ?")
            .bind(kind)
            .bind(limit as i64)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as("SELECT id, kind, name, source FROM entities ORDER BY created_at DESC LIMIT ?")
            .bind(limit as i64)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| e.to_string())?
    };

    let items: Vec<serde_json::Value> = rows.into_iter().map(|(id, kind, name, source)| {
        json!({ "id": id, "kind": kind, "name": name, "source": source })
    }).collect();

    Ok(Json(json!({ "entities": items })))
}

pub async fn relations(
    State(state): State<super::AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, String> {
    let limit = params.limit.unwrap_or(20).min(100);
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, from_id, to_id, kind FROM relations ORDER BY created_at DESC LIMIT ?"
    )
    .bind(limit as i64)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let items: Vec<serde_json::Value> = rows.into_iter().map(|(id, from_id, to_id, kind)| {
        json!({ "id": id, "from_id": from_id, "to_id": to_id, "kind": kind })
    }).collect();

    Ok(Json(json!({ "relations": items })))
}
