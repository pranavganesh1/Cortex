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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Commit => write!(f, "Commit"),
            EntityKind::File => write!(f, "File"),
            EntityKind::Function => write!(f, "Function"),
            EntityKind::Struct => write!(f, "Struct"),
            EntityKind::Class => write!(f, "Class"),
            EntityKind::Enum => write!(f, "Enum"),
            EntityKind::Trait => write!(f, "Trait"),
            EntityKind::Interface => write!(f, "Interface"),
            EntityKind::Module => write!(f, "Module"),
            EntityKind::Decision => write!(f, "Decision"),
            EntityKind::Note => write!(f, "Note"),
        }
    }
}

impl EntityKind {
    /// Returns a short icon/symbol for compact display in tables
    pub fn icon(&self) -> &'static str {
        match self {
            EntityKind::Function => "ƒ",
            EntityKind::Struct => "S",
            EntityKind::Class => "C",
            EntityKind::Enum => "E",
            EntityKind::Trait => "T",
            EntityKind::Interface => "I",
            EntityKind::File => "📄",
            EntityKind::Commit => "📝",
            EntityKind::Module => "📦",
            EntityKind::Decision => "💡",
            EntityKind::Note => "📌",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub kind: RelationKind,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    Authored,
    Modified,
    Contains,
    DependsOn,
    DecidedIn,
}

impl std::fmt::Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationKind::Authored => write!(f, "Authored"),
            RelationKind::Modified => write!(f, "Modified"),
            RelationKind::Contains => write!(f, "Contains"),
            RelationKind::DependsOn => write!(f, "DependsOn"),
            RelationKind::DecidedIn => write!(f, "DecidedIn"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub entity_id: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<String>,
}
