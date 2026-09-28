use crate::core::models::{Entity, EntityKind, Relation, RelationKind};
use crate::extract::decisions::{extract_from_text, store_decisions, SourceKind};
use chrono::{TimeZone, Utc};
use git2::Repository;
use anyhow::Result;
use sqlx::SqlitePool;

/// Ingests a Git repository by analyzing its commit history and converting it to entities.
pub async fn ingest_repo(path: &str, pool: &SqlitePool) -> Result<(Vec<Entity>, Vec<Relation>)> {
    let repo = Repository::open(path)?;
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    for oid in revwalk.take(1000) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        
        let commit_id = format!("commit:{}", oid);
        let author = commit.author();
        let author_name = author.name().unwrap_or("unknown").to_string();
        let author_id = format!("author:{}", author_name);
        let message = commit.message().unwrap_or("").to_string();
        let time = Utc.timestamp_opt(commit.time().seconds(), 0).expect("Failed to parse git commit timestamp");

        // Author entity
        entities.push(Entity {
            id: author_id.clone(),
            kind: EntityKind::Note,
            name: author_name,
            content: None,
            created_at: time,
            updated_at: time,
            source: "git:author".to_string(),
            parent_id: None,
        });

        // Commit entity
        entities.push(Entity {
            id: commit_id.clone(),
            kind: EntityKind::Commit,
            name: message.lines().next().unwrap_or("").to_string(),
            content: Some(message.clone()),
            created_at: time,
            updated_at: time,
            source: "git:commit".to_string(),
            parent_id: None,
        });

        // Relation: author -> commit
        relations.push(Relation {
            id: format!("rel:{}:{}", author_id, commit_id),
            from_id: author_id,
            to_id: commit_id.clone(),
            kind: RelationKind::Authored,
            created_at: time,
        });

        // Extract decisions from commit message
        let decisions = extract_from_text(&message, &commit_id, SourceKind::CommitMessage);
        store_decisions(pool, &decisions, Some(commit_id.clone())).await?;

        // Log commit event
        crate::core::events::log_event(pool, &commit_id, "git:commit", Some(message.lines().next().unwrap_or(""))).await?;

        // Push to working memory
        crate::memory::stack::add_commit(&oid.to_string());

        // Files in this commit
        let tree = commit.tree()?;
        for entry in tree.iter() {
            if let Some(name) = entry.name() {
                let file_id = format!("file:{}:{}", oid, name);
                entities.push(Entity {
                    id: file_id.clone(),
                    kind: EntityKind::File,
                    name: name.to_string(),
                    content: None,
                    created_at: time,
                    updated_at: time,
                    source: "git:file".to_string(),
                    parent_id: Some(commit_id.clone()),
                });

                relations.push(Relation {
                    id: format!("rel:{}:{}", commit_id, file_id),
                    from_id: commit_id.clone(),
                    to_id: file_id,
                    kind: RelationKind::Modified,
                    created_at: time,
                });
            }
        }
    }

    Ok((entities, relations))
}

