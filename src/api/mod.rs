pub mod routes;
pub mod context;

use axum::{Router, routing::get, http::Method};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub async fn start_server(pool: SqlitePool) -> anyhow::Result<()> {
    let state = AppState { pool };
    
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);
    
    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/status", get(routes::status))
        .route("/context", get(routes::context))
        .route("/active-context", get(routes::active_context))
        .route("/focus", get(routes::focus))
        .layer(cors)
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787").await?;
    println!("🧠 Cortex API listening on http://127.0.0.1:8787");
    
    axum::serve(listener, app).await?;
    Ok(())
}
