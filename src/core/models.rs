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
    pub source: String,
    pub parent_id: Option<String>, // which file/commit owns this entity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityKind {
    Commit,
    File,
    Function,
    Struct,
    Class,
    Enum,
    Trait,
    Interface,
    Module,
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
    Authored,
    Modified,
    Contains,
    DependsOn,
    DecidedIn,
}
