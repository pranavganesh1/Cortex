use axum::{extract::{Query, State}, response::Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ContextParams {
    file_path: String,
    query: String,
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
