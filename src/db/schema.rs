use sqlx::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS entities (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            source TEXT NOT NULL,
            parent_id TEXT
        );

        CREATE TABLE IF NOT EXISTS relations (
            id TEXT PRIMARY KEY,
            from_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            action TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            metadata TEXT
        );

        CREATE TABLE IF NOT EXISTS embeddings (
            entity_id TEXT PRIMARY KEY,
            vector TEXT NOT NULL,
            model TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_entities_kind ON entities(kind);
        CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
        CREATE INDEX IF NOT EXISTS idx_entities_parent ON entities(parent_id);
        CREATE INDEX IF NOT EXISTS idx_relations_from ON relations(from_id);
        CREATE INDEX IF NOT EXISTS idx_relations_to ON relations(to_id);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        CREATE INDEX IF NOT EXISTS idx_events_entity ON events(entity_id);
        CREATE INDEX IF NOT EXISTS idx_events_action ON events(action);
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}
