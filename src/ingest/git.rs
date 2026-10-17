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
        let time = Utc.timestamp_opt(commit.time().seconds(), 0).single().unwrap_or_else(Utc::now);

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

        // Files modified in this commit (diff against parent, or empty tree if initial commit)
        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
        for delta in diff.deltas() {
            let path_opt = delta.new_file().path().or_else(|| delta.old_file().path());
            if let Some(path) = path_opt {
                let path_str = path.to_string_lossy().replace('\\', "/");
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or(&path_str).to_string();
                let file_id = format!("file:{}:{}", oid, path_str);

                entities.push(Entity {
                    id: file_id.clone(),
                    kind: EntityKind::File,
                    name: file_name,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use git2::{Signature, Time};

    #[tokio::test]
    async fn test_ingest_repo_extracts_commits_and_modified_files() {
        let temp_dir = std::env::temp_dir().join(format!("cortex_git_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let repo = Repository::init(&temp_dir).unwrap();
        let sig = Signature::new("Test User", "test@example.com", &Time::new(1700000000, 0)).unwrap();

        // Create a file in a subdirectory
        let sub_dir = temp_dir.join("src");
        std::fs::create_dir_all(&sub_dir).unwrap();
        let file_path = sub_dir.join("lib.rs");
        {
            let mut f = File::create(&file_path).unwrap();
            writeln!(f, "// DECISION: choose sqlite for persistence").unwrap();
        }

        // Add and commit
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("src/lib.rs")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let commit_oid = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit: setup sqlite database",
            &tree,
            &[],
        ).unwrap();

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::db::schema::init_db(&pool).await.unwrap();

        let (entities, relations) = ingest_repo(temp_dir.to_str().unwrap(), &pool).await.unwrap();

        assert!(entities.iter().any(|e| e.kind == EntityKind::Commit && e.id.contains(&commit_oid.to_string())));
        assert!(entities.iter().any(|e| e.kind == EntityKind::File && e.name == "lib.rs"));
        assert!(relations.iter().any(|r| r.kind == RelationKind::Authored));
        assert!(relations.iter().any(|r| r.kind == RelationKind::Modified));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}


