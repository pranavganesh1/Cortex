use super::{ParsedQuery, QueryIntent, RelationDirection, RelationQuery};
use crate::core::models::{EntityKind, RelationKind};
use regex::Regex;

pub fn parse_query(input: &str) -> ParsedQuery {
    let lower = input.to_lowercase();

    // 1. Detect intent
    let intent = if lower.starts_with("how many") || lower.starts_with("count") {
        QueryIntent::Count
    } else if lower.starts_with("find") || lower.starts_with("search") {
        QueryIntent::Find
    } else if lower.starts_with("show") || lower.starts_with("list") || lower.starts_with("what") {
        QueryIntent::List
    } else if lower.starts_with("why") || lower.starts_with("explain") {
        QueryIntent::Explain
    } else {
        QueryIntent::List
    };

    // 2. Detect entity kind
    let entity_kind = detect_entity_kind(&lower);

    // 3. Detect location ("in src/main.rs", "in the database module")
    let location = detect_location(&lower);

    // 4. Detect name pattern ("named handle_*", "called User")
    let name_pattern = detect_name_pattern(&lower);

    // 5. Detect content search ("about authentication", "handling errors")
    let content_pattern = detect_content_pattern(&lower);

    // 6. Detect relations ("contains", "depends on", "authored by")
    let relation = detect_relation(&lower);

    ParsedQuery {
        intent,
        entity_kind,
        name_pattern,
        content_pattern,
        location,
        relation,
        limit: 50,
    }
}

fn detect_entity_kind(text: &str) -> Option<EntityKind> {
    if text.contains("function") || text.contains("functions") || text.contains("fn ") || text.contains("methods") {
        Some(EntityKind::Function)
    } else if text.contains("struct") || text.contains("structs") || text.contains("structures") {
        Some(EntityKind::Struct)
    } else if text.contains("class") || text.contains("classes") {
        Some(EntityKind::Class)
    } else if text.contains("enum") || text.contains("enums") {
        Some(EntityKind::Enum)
    } else if text.contains("trait") || text.contains("traits") || text.contains("interface") || text.contains("interfaces") {
        Some(EntityKind::Trait)
    } else if text.contains("file") || text.contains("files") {
        Some(EntityKind::File)
    } else if text.contains("commit") || text.contains("commits") {
        Some(EntityKind::Commit)
    } else if text.contains("module") || text.contains("modules") {
        Some(EntityKind::Module)
    } else {
        None
    }
}

fn detect_location(text: &str) -> Option<String> {
    // "in src/main.rs", "in file src/main.rs", "in the database module"
    let re = Regex::new(r#"\bin\s+(?:file\s+|the\s+)?([a-zA-Z0-9_./\-]+)"#).ok()?;
    re.captures(text).map(|cap| cap.get(1).unwrap().as_str().to_string())
}

fn detect_name_pattern(text: &str) -> Option<String> {
    // "named handle_request", "called User", "named handle_*"
    let re = Regex::new(r#"(?:named|called)\s+([a-zA-Z0-9_*]+)"#).ok()?;
    re.captures(text).map(|cap| {
        let raw = cap.get(1).unwrap().as_str().to_string();
        // Convert * to SQL wildcard %
        raw.replace('*', "%")
    })
}

fn detect_content_pattern(text: &str) -> Option<String> {
    // "about authentication", "handling errors", "related to payment"
    let re = Regex::new(r#"(?:about|handling|related to|for|that handles|that does)\s+([a-zA-Z0-9_ ]+)"#).ok()?;
    re.captures(text).map(|cap| cap.get(1).unwrap().as_str().to_string())
}

fn detect_relation(text: &str) -> Option<RelationQuery> {
    if text.contains("contains") || text.contains("has") || text.contains("includes") {
        Some(RelationQuery {
            kind: RelationKind::Contains,
            direction: RelationDirection::From,
            target_name: None,
        })
    } else if text.contains("depends on") || text.contains("uses") || text.contains("requires") {
        Some(RelationQuery {
            kind: RelationKind::DependsOn,
            direction: RelationDirection::From,
            target_name: None,
        })
    } else if text.contains("authored by") || text.contains("written by") || text.contains("by") {
        Some(RelationQuery {
            kind: RelationKind::Authored,
            direction: RelationDirection::To,
            target_name: None,
        })
    } else {
        None
    }
}
