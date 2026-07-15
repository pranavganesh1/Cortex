pub mod routes;
pub mod context;

use axum::{Router, routing::get};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub async fn start_server(pool: SqlitePool) -> anyhow::Result<()> {
    let state = AppState { pool };
    
    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/status", get(routes::status))
        .route("/context", get(routes::context))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787").await?;
    println!("🧠 Cortex API listening on http://127.0.0.1:8787");
    
    axum::serve(listener, app).await?;
    Ok(())
}
