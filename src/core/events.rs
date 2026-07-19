use crate::core::models::Event;
use chrono::Utc;
use sqlx::SqlitePool;
use anyhow::Result;
use uuid::Uuid;

pub async fn log_event(pool: &SqlitePool, entity_id: &str, action: &str, metadata: Option<&str>) -> Result<()> {
    let event = Event {
        id: format!("evt:{}", Uuid::new_v4()),
        entity_id: entity_id.to_string(),
        action: action.to_string(),
        timestamp: Utc::now(),
        metadata: metadata.map(|s| s.to_string()),
    };

    sqlx::query(
        "INSERT INTO events (id, entity_id, action, timestamp, metadata) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&event.id)
    .bind(&event.entity_id)
    .bind(&event.action)
    .bind(event.timestamp.to_rfc3339())
    .bind(&event.metadata)
    .execute(pool)
    .await?;

    Ok(())
}
