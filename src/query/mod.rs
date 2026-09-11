// Query module definitions
pub mod parser;
pub mod executor;
pub mod formatter;

use crate::core::models::{EntityKind, RelationKind};


/// Represents a parsed natural language query translated into query parameters.
#[derive(Debug, Clone)]
pub struct ParsedQuery {
    pub intent: QueryIntent,
    pub entity_kind: Option<EntityKind>,
    pub name_pattern: Option<String>,
    pub content_pattern: Option<String>,
    pub location: Option<String>,      // e.g., "src/main.rs"
    pub relation: Option<RelationQuery>,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryIntent {
    List,
    Find,
    Show,
    Count,
    Explain,
}

impl std::fmt::Display for QueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryIntent::List => write!(f, "list"),
            QueryIntent::Find => write!(f, "find"),
            QueryIntent::Show => write!(f, "show"),
            QueryIntent::Count => write!(f, "count"),
            QueryIntent::Explain => write!(f, "explain"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationQuery {
    pub kind: RelationKind,
    pub direction: RelationDirection,
    pub target_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationDirection {
    From, // "what X contains Y"
    To,   // "what X is contained in Y"
}

impl std::fmt::Display for RelationDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationDirection::From => write!(f, "from"),
            RelationDirection::To => write!(f, "to"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_intent_display() {
        assert_eq!(QueryIntent::List.to_string(), "list");
        assert_eq!(QueryIntent::Find.to_string(), "find");
        assert_eq!(QueryIntent::Show.to_string(), "show");
        assert_eq!(QueryIntent::Count.to_string(), "count");
        assert_eq!(QueryIntent::Explain.to_string(), "explain");
    }

    #[test]
    fn test_relation_direction_display_and_equality() {
        assert_eq!(RelationDirection::From.to_string(), "from");
        assert_eq!(RelationDirection::To.to_string(), "to");
        assert_eq!(RelationDirection::From, RelationDirection::From);
        assert_ne!(RelationDirection::From, RelationDirection::To);
    }
}
