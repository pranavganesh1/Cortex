use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub kind: EntityKind,
    pub name: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: String, // e.g., "git:commit", "git:file", "manual"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityKind {
    Commit,
    File,
    Function,
    Decision,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub kind: RelationKind,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationKind {
    Authored,      // person -> commit
    Modified,      // commit -> file
    Contains,      // file -> function
    DependsOn,     // function -> function
    DecidedIn,     // decision -> commit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub entity_id: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<String>,
}
