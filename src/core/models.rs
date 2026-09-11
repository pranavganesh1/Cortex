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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_kind_display_and_icon() {
        assert_eq!(EntityKind::Function.to_string(), "Function");
        assert_eq!(EntityKind::Function.icon(), "ƒ");
        assert_eq!(EntityKind::Struct.to_string(), "Struct");
        assert_eq!(EntityKind::Struct.icon(), "S");
        assert_eq!(EntityKind::Class.to_string(), "Class");
        assert_eq!(EntityKind::Class.icon(), "C");
        assert_eq!(EntityKind::Enum.to_string(), "Enum");
        assert_eq!(EntityKind::Enum.icon(), "E");
        assert_eq!(EntityKind::Trait.to_string(), "Trait");
        assert_eq!(EntityKind::Trait.icon(), "T");
        assert_eq!(EntityKind::Interface.to_string(), "Interface");
        assert_eq!(EntityKind::Interface.icon(), "I");
        assert_eq!(EntityKind::File.to_string(), "File");
        assert_eq!(EntityKind::File.icon(), "📄");
        assert_eq!(EntityKind::Commit.to_string(), "Commit");
        assert_eq!(EntityKind::Commit.icon(), "📝");
        assert_eq!(EntityKind::Module.to_string(), "Module");
        assert_eq!(EntityKind::Module.icon(), "📦");
        assert_eq!(EntityKind::Decision.to_string(), "Decision");
        assert_eq!(EntityKind::Decision.icon(), "💡");
        assert_eq!(EntityKind::Note.to_string(), "Note");
        assert_eq!(EntityKind::Note.icon(), "📌");
    }

    #[test]
    fn test_relation_kind_display() {
        assert_eq!(RelationKind::Authored.to_string(), "Authored");
        assert_eq!(RelationKind::Modified.to_string(), "Modified");
        assert_eq!(RelationKind::Contains.to_string(), "Contains");
        assert_eq!(RelationKind::DependsOn.to_string(), "DependsOn");
        assert_eq!(RelationKind::DecidedIn.to_string(), "DecidedIn");
    }
}
