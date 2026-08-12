use super::{ParsedQuery, QueryIntent, RelationDirection, RelationQuery};
use crate::core::models::{EntityKind, RelationKind};
use regex::Regex;

pub fn parse_query(input: &str) -> ParsedQuery {
    let lower = input.to_lowercase();

    // 1. Detect intent
    let intent = if lower.starts_with("why") || lower.starts_with("how come") || lower.contains("reason") || lower.starts_with("explain") {
        QueryIntent::Explain
    } else if lower.starts_with("how many") || lower.starts_with("count") {
        QueryIntent::Count
    } else if lower.starts_with("find") || lower.starts_with("search") {
        QueryIntent::Find
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

    // 7. Detect limit ("top 10", "limit 5", "first 20")
    let limit = detect_limit(&lower).unwrap_or(50);

    ParsedQuery {
        intent,
        entity_kind,
        name_pattern,
        content_pattern,
        location,
        relation,
        limit,
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

fn detect_limit(text: &str) -> Option<usize> {
    let re = Regex::new(r#"\b(?:limit|top|first|max)\s+(\d+)\b"#).ok()?;
    let cap = re.captures(text)?;
    cap.get(1)?.as_str().parse::<usize>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_intents() {
        let q1 = parse_query("why did authentication fail?");
        assert_eq!(q1.intent, QueryIntent::Explain);

        let q2 = parse_query("how many functions in src/main.rs");
        assert_eq!(q2.intent, QueryIntent::Count);
        assert_eq!(q2.entity_kind, Some(EntityKind::Function));

        let q3 = parse_query("find struct named User");
        assert_eq!(q3.intent, QueryIntent::Find);
        assert_eq!(q3.entity_kind, Some(EntityKind::Struct));
        assert_eq!(q3.name_pattern, Some("user".to_string()));
    }

    #[test]
    fn test_parse_query_wildcards_and_locations() {
        let q = parse_query("find function named handle_* in src/api");
        assert_eq!(q.entity_kind, Some(EntityKind::Function));
        assert_eq!(q.name_pattern, Some("handle_%".to_string()));
        assert_eq!(q.location, Some("src/api".to_string()));
    }

    #[test]
    fn test_parse_query_limit() {
        let q1 = parse_query("find top 10 functions in src/api");
        assert_eq!(q1.limit, 10);

        let q2 = parse_query("list first 5 commits");
        assert_eq!(q2.limit, 5);

        let q3 = parse_query("find functions limit 15");
        assert_eq!(q3.limit, 15);

        let q4 = parse_query("find struct named User");
        assert_eq!(q4.limit, 50);
    }
}
