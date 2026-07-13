pub mod connection;
pub mod schema;

use sqlx::SqlitePool;
use crate::core::models::{Entity, Relation};

pub async fn insert_entities(pool: &SqlitePool, entities: &[Entity]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    for e in entities {
        sqlx::query(
            r#"INSERT OR IGNORE INTO entities 
               (id, kind, name, content, created_at, updated_at, source, parent_id) 
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(&e.id)
        .bind(format!("{:?}", e.kind))
        .bind(&e.name)
        .bind(&e.content)
        .bind(e.created_at.to_rfc3339())
        .bind(e.updated_at.to_rfc3339())
        .bind(&e.source)
        .bind(&e.parent_id)
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

pub async fn upsert_entity(pool: &SqlitePool, entity: &Entity) -> anyhow::Result<()> {
    let updated = sqlx::query(
        "UPDATE entities SET kind = ?, name = ?, content = ?, updated_at = ?, source = ?, parent_id = ? WHERE id = ?"
    )
    .bind(format!("{:?}", entity.kind))
    .bind(&entity.name)
    .bind(&entity.content)
    .bind(entity.updated_at.to_rfc3339())
    .bind(&entity.source)
    .bind(&entity.parent_id)
    .bind(&entity.id)
    .execute(pool)
    .await?
    .rows_affected();

    if updated == 0 {
        sqlx::query(
            "INSERT INTO entities (id, kind, name, content, created_at, updated_at, source, parent_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&entity.id)
        .bind(format!("{:?}", entity.kind))
        .bind(&entity.name)
        .bind(&entity.content)
        .bind(entity.created_at.to_rfc3339())
        .bind(entity.updated_at.to_rfc3339())
        .bind(&entity.source)
        .bind(&entity.parent_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// Delete all code entities belonging to a file before re-parsing
pub async fn delete_entities_for_file(pool: &SqlitePool, file_id: &str) -> anyhow::Result<()> {
    // Delete relations involving this file's code entities
    sqlx::query(
        r#"DELETE FROM relations 
           WHERE from_id IN (SELECT id FROM entities WHERE parent_id = ?)
           OR to_id IN (SELECT id FROM entities WHERE parent_id = ?)"#
    )
    .bind(file_id)
    .bind(file_id)
    .execute(pool)
    .await?;

    // Delete the code entities themselves
    sqlx::query("DELETE FROM entities WHERE parent_id = ?")
        .bind(file_id)
        .execute(pool)
        .await?;

    Ok(())
}
