use axum::{extract::{Query, State}, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContextParams {
    file_path: String,
    query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let Json(val) = health().await;
        assert_eq!(val["status"], "ok");
    }

    #[test]
    fn test_deserialize_context_params() {
        let json_str = r#"{"file_path":"src/main.rs","query":"explain"}"#;
        let params: ContextParams = serde_json::from_str(json_str).unwrap();
        assert_eq!(params.file_path, "src/main.rs");
        assert_eq!(params.query, "explain");
    }

    #[test]
    fn test_deserialize_active_context_params() {
        let json_with_query = r#"{"query":"how does this work?"}"#;
        let params: ActiveContextParams = serde_json::from_str(json_with_query).unwrap();
        assert_eq!(params.query.as_deref(), Some("how does this work?"));

        let empty_json = r#"{}"#;
        let empty_params: ActiveContextParams = serde_json::from_str(empty_json).unwrap();
        assert_eq!(empty_params.query, None);
    }

    #[test]
    fn test_deserialize_list_params() {
        let json_str = r#"{"limit": 50, "kind": "Commit"}"#;
        let params: ListParams = serde_json::from_str(json_str).unwrap();
        assert_eq!(params.limit, Some(50));
        assert_eq!(params.kind.as_deref(), Some("Commit"));

        let empty: ListParams = serde_json::from_str("{}").unwrap();
        assert_eq!(empty.limit, None);
        assert_eq!(empty.kind, None);
    }
}

