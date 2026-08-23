use super::{ParsedQuery, QueryIntent, RelationDirection};
use crate::core::models::{Entity, EntityKind, Relation};
use sqlx::SqlitePool;
use anyhow::Result;

/// Executes a parsed query against the SQLite database.
pub async fn execute_query(pool: &SqlitePool, query: &ParsedQuery) -> Result<QueryResult> {
    match query.intent {
        QueryIntent::Count => execute_count(pool, query).await,
        QueryIntent::List | QueryIntent::Find | QueryIntent::Show => execute_list(pool, query).await,
        QueryIntent::Explain => execute_explain(pool, query).await,
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub total_count: i64,
    pub query_description: String,
}

async fn execute_list(pool: &SqlitePool, query: &ParsedQuery) -> Result<QueryResult> {
    let mut sql = String::from("SELECT id, kind, name, content, created_at, updated_at, source, parent_id FROM entities WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();
    let mut desc = String::new();

    // Entity kind filter (safe — value comes from our enum via Display)
    if let Some(kind) = &query.entity_kind {
        sql.push_str(" AND kind = ?");
        binds.push(format!("{}", kind));
        desc.push_str(&format!("{}s", kind));
    } else {
        desc.push_str("entities");
    }

    // Name pattern (user-derived — must be parameterized)
    if let Some(pattern) = &query.name_pattern {
        sql.push_str(" AND name LIKE ?");
        binds.push(format!("%{}%", pattern));
        desc.push_str(&format!(" matching '{}'", pattern));
    }

    // Content search (user-derived — must be parameterized)
    if let Some(content) = &query.content_pattern {
        sql.push_str(" AND (name LIKE ? OR content LIKE ?)");
        binds.push(format!("%{}%", content));
        binds.push(format!("%{}%", content));
        desc.push_str(&format!(" related to '{}'", content));
    }

    // Location filter
    if let Some(loc) = &query.location {
        let file_id = if loc.starts_with("file:") {
            loc.clone()
        } else {
            format!("file:{}", loc)
        };
        sql.push_str(" AND (parent_id = ? OR id = ?)");
        binds.push(file_id.clone());
        binds.push(file_id);
        desc.push_str(&format!(" in '{}'", loc));
    }

    sql.push_str(&format!(" LIMIT {}", query.limit));

    // Build and execute the query with bind parameters
    let mut db_query = sqlx::query_as::<_, EntityRow>(&sql);
    for val in &binds {
        db_query = db_query.bind(val);
    }
    let rows = db_query.fetch_all(pool).await?;

    let entities: Vec<Entity> = rows.into_iter().map(|r| Entity {
        id: r.id,
        kind: parse_kind(&r.kind),
        name: r.name,
        content: r.content,
        created_at: r.created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: r.updated_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
        source: r.source,
        parent_id: r.parent_id,
    }).collect();

    // If relation query, traverse using parameterized queries
    let mut relations = Vec::new();
    if let Some(rel) = &query.relation {
        for entity in &entities {
            let (rel_sql, bind_id) = match rel.direction {
                RelationDirection::From => (
                    "SELECT id, from_id, to_id, kind, created_at FROM relations WHERE from_id = ? AND kind = ?",
                    &entity.id,
                ),
                RelationDirection::To => (
                    "SELECT id, from_id, to_id, kind, created_at FROM relations WHERE to_id = ? AND kind = ?",
                    &entity.id,
                ),
            };
            let rel_rows = sqlx::query_as::<_, RelationRow>(rel_sql)
                .bind(bind_id)
                .bind(format!("{}", rel.kind))
                .fetch_all(pool)
                .await?;
            for r in rel_rows {
                relations.push(Relation {
                    id: r.id,
                    from_id: r.from_id,
                    to_id: r.to_id,
                    kind: parse_relation_kind(&r.kind),
                    created_at: r.created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
                });
            }
        }
    }

    let total_count = entities.len() as i64;

    Ok(QueryResult {
        entities,
        relations,
        total_count,
        query_description: desc,
    })
}

async fn execute_count(pool: &SqlitePool, query: &ParsedQuery) -> Result<QueryResult> {
    let mut sql = String::from("SELECT COUNT(*) FROM entities WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();

    if let Some(kind) = &query.entity_kind {
        sql.push_str(" AND kind = ?");
        binds.push(format!("{}", kind));
    }
    if let Some(pattern) = &query.name_pattern {
        sql.push_str(" AND name LIKE ?");
        binds.push(format!("%{}%", pattern));
    }
    if let Some(content) = &query.content_pattern {
        sql.push_str(" AND (name LIKE ? OR content LIKE ?)");
        binds.push(format!("%{}%", content));
        binds.push(format!("%{}%", content));
    }
    if let Some(loc) = &query.location {
        let file_id = if loc.starts_with("file:") { loc.clone() } else { format!("file:{}", loc) };
        sql.push_str(" AND (parent_id = ? OR id = ?)");
        binds.push(file_id.clone());
        binds.push(file_id);
    }

    let mut db_query = sqlx::query_scalar::<_, i64>(&sql);
    for val in &binds {
        db_query = db_query.bind(val);
    }
    let count = db_query.fetch_one(pool).await?;

    Ok(QueryResult {
        entities: vec![],
        relations: vec![],
        total_count: count,
        query_description: "count of matching entities".to_string(),
    })
}

async fn execute_explain(pool: &SqlitePool, query: &ParsedQuery) -> Result<QueryResult> {
    // Search for decisions related to the query topic
    let topic = query.content_pattern.as_ref()
        .or(query.name_pattern.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");
    
    let decisions: Vec<EntityRow> = sqlx::query_as(
        r#"SELECT id, kind, name, content, created_at, updated_at, source, parent_id FROM entities 
           WHERE kind = 'Decision' 
           AND (name LIKE ? OR content LIKE ?)
           ORDER BY created_at DESC
           LIMIT 10"#
    )
    .bind(format!("%{}%", topic))
    .bind(format!("%{}%", topic))
    .fetch_all(pool)
    .await?;
    
    let entities: Vec<Entity> = decisions.into_iter().map(|r| Entity {
        id: r.id,
        kind: parse_kind(&r.kind),
        name: r.name,
        content: r.content,
        created_at: r.created_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: r.updated_at.parse().unwrap_or_else(|_| chrono::Utc::now()),
        source: r.source,
        parent_id: r.parent_id,
    }).collect();
    
    let total_count = entities.len() as i64;
    
    Ok(QueryResult {
        entities,
        relations: vec![],
        total_count,
        query_description: format!("decisions about '{}'", topic),
    })
}

// SQLx row helpers
#[derive(sqlx::FromRow)]
struct EntityRow {
    id: String,
    kind: String,
    name: String,
    content: Option<String>,
    created_at: String,
    updated_at: String,
    source: String,
    parent_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct RelationRow {
    id: String,
    from_id: String,
    to_id: String,
    kind: String,
    created_at: String,
}

fn parse_kind(s: &str) -> EntityKind {
    match s {
        "Commit" => EntityKind::Commit,
        "File" => EntityKind::File,
        "Function" => EntityKind::Function,
        "Struct" => EntityKind::Struct,
        "Class" => EntityKind::Class,
        "Enum" => EntityKind::Enum,
        "Trait" => EntityKind::Trait,
        "Interface" => EntityKind::Interface,
        "Module" => EntityKind::Module,
        "Decision" => EntityKind::Decision,
        "Note" => EntityKind::Note,
        _ => EntityKind::Note,
    }
}

fn parse_relation_kind(s: &str) -> crate::core::models::RelationKind {
    match s {
        "Authored" => crate::core::models::RelationKind::Authored,
        "Modified" => crate::core::models::RelationKind::Modified,
        "Contains" => crate::core::models::RelationKind::Contains,
        "DependsOn" => crate::core::models::RelationKind::DependsOn,
        "DecidedIn" => crate::core::models::RelationKind::DecidedIn,
        _ => crate::core::models::RelationKind::Contains,
    }
}
