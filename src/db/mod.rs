pub mod connection;
pub mod schema;

use sqlx::SqlitePool;
use crate::core::models::{Entity, Relation};

pub async fn insert_entities(pool: &SqlitePool, entities: &[Entity]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    for e in entities {
        sqlx::query(
            r#"INSERT OR IGNORE INTO entities 
               (id, kind, name, content, created_at, updated_at, source) 
               VALUES (?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&e.id)
        .bind(format!("{:?}", e.kind))
        .bind(&e.name)
        .bind(&e.content)
        .bind(e.created_at.to_rfc3339())
        .bind(e.updated_at.to_rfc3339())
        .bind(&e.source)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn insert_relations(pool: &SqlitePool, relations: &[Relation]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    for r in relations {
        sqlx::query(
            r#"INSERT OR IGNORE INTO relations 
               (id, from_id, to_id, kind, created_at) 
               VALUES (?, ?, ?, ?, ?)"#
        )
        .bind(&r.id)
        .bind(&r.from_id)
        .bind(&r.to_id)
        .bind(format!("{:?}", r.kind))
        .bind(r.created_at.to_rfc3339())
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
