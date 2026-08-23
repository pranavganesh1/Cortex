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

#[derive(Debug, Clone)]
pub struct RelationQuery {
    pub kind: RelationKind,
    pub direction: RelationDirection,
    pub target_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RelationDirection {
    From, // "what X contains Y"
    To,   // "what X is contained in Y"
}
