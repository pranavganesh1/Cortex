use crate::core::models::Event;
use chrono::Utc;
use sqlx::SqlitePool;
use anyhow::Result;
use uuid::Uuid;

pub fn create_event(entity_id: &str, action: &str, metadata: Option<&str>) -> Event {
    Event {
        id: format!("evt:{}", Uuid::new_v4()),
        entity_id: entity_id.to_string(),
        action: action.to_string(),
        timestamp: Utc::now(),
        metadata: metadata.map(|s| s.to_string()),
    }
}

pub async fn log_event(pool: &SqlitePool, entity_id: &str, action: &str, metadata: Option<&str>) -> Result<()> {
    let event = create_event(entity_id, action, metadata);

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

pub async fn log_events_batch(pool: &SqlitePool, items: &[(&str, &str, Option<&str>)]) -> Result<usize> {
    if items.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;
    let mut count = 0;

    for (entity_id, action, metadata) in items {
        let event = create_event(entity_id, action, *metadata);
        sqlx::query(
            "INSERT INTO events (id, entity_id, action, timestamp, metadata) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&event.id)
        .bind(&event.entity_id)
        .bind(&event.action)
        .bind(event.timestamp.to_rfc3339())
        .bind(&event.metadata)
        .execute(&mut *tx)
        .await?;
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_event() {
        let evt = create_event("ent:123", "update", Some("{\"key\":\"val\"}"));
        assert!(evt.id.starts_with("evt:"));
        assert_eq!(evt.entity_id, "ent:123");
        assert_eq!(evt.action, "update");
        assert_eq!(evt.metadata, Some("{\"key\":\"val\"}".to_string()));
    }
}
