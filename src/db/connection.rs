use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

pub async fn connect(db_path: &Path) -> anyhow::Result<SqlitePool> {
    let url = format!("sqlite:{}", db_path.display());
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    Ok(pool)
}
