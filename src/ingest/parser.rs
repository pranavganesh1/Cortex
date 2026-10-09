use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};
use streaming_iterator::StreamingIterator;
use crate::core::models::{Entity, EntityKind, Relation, RelationKind};
use chrono::Utc;

const RUST_QUERY: &str = r#"
    (function_item name: (identifier) @func)
    (struct_item name: (type_identifier) @struct)
    (enum_item name: (type_identifier) @enum)
    (trait_item name: (type_identifier) @trait)
    (impl_item type: (type_identifier) @impl)
"#;

const PYTHON_QUERY: &str = r#"
    (function_definition name: (identifier) @func)
    (class_definition name: (identifier) @class)
"#;

const JS_QUERY: &str = r#"
    (function_declaration name: (identifier) @func)
    (class_declaration name: (identifier) @class)
    (method_definition name: (property_identifier) @method)
"#;

const TS_QUERY: &str = r#"
    (function_declaration name: (identifier) @func)
    (class_declaration name: (type_identifier) @class)
    (method_definition name: (property_identifier) @method)
    (interface_declaration name: (type_identifier) @interface)
    (type_alias_declaration name: (type_identifier) @type)
"#;

const GO_QUERY: &str = r#"
    (function_declaration name: (identifier) @func)
    (method_declaration name: (field_identifier) @method)
    (type_spec name: (type_identifier) @type)
"#;

/// Parses the code file into entities and structural relationships using tree-sitter.
/// Returns `None` if the file extension is not supported or parsing fails.
pub fn parse_file(path: &Path, content: &str) -> Option<(Vec<Entity>, Vec<Relation>)> {
    let ext = path.extension()?.to_str()?;
    let (language, query_str): (Language, &str) = match ext {
        "rs" => (tree_sitter_rust::LANGUAGE.into(), RUST_QUERY),
        "py" => (tree_sitter_python::LANGUAGE.into(), PYTHON_QUERY),
        "js" | "jsx" => (tree_sitter_javascript::LANGUAGE.into(), JS_QUERY),
        "ts" | "tsx" => {
            let lang: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
            (lang, TS_QUERY)
        }
        "go" => (tree_sitter_go::LANGUAGE.into(), GO_QUERY),
        _ => return None,
    };

    let mut parser = Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(content, None)?;
    let root = tree.root_node();

    let query = Query::new(&language, query_str).ok()?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root, content.as_bytes());

    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let file_id = format!("file:{}", path.to_string_lossy());
    let capture_names: Vec<&str> = query.capture_names().iter().map(|s| s.as_ref()).collect();

    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            let text = &content[node.byte_range()];
            let capture_name = match capture_names.get(capture.index as usize) {
                Some(name) => *name,
                None => continue,
            };
            let line = node.start_position().row;

            let (kind, prefix) = match capture_name {
                "func" | "method" => (EntityKind::Function, "func"),
                "struct" | "class" | "type" | "impl" => (EntityKind::Struct, "struct"),
                "enum" => (EntityKind::Enum, "enum"),
                "trait" | "interface" => (EntityKind::Trait, "trait"),
                _ => continue,
            };

            let entity_id = format!("{}:{}:{}@{}", prefix, path.to_string_lossy(), text, line);

            let entity = Entity {
                id: entity_id.clone(),
                kind,
                name: text.to_string(),
                content: Some(text.to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                source: "tree-sitter".to_string(),
                parent_id: Some(file_id.clone()),
            };

            entities.push(entity);

            let relation = Relation {
                id: format!("rel:contains:{}:{}", file_id, entity_id),
                from_id: file_id.clone(),
                to_id: entity_id,
                kind: RelationKind::Contains,
                created_at: Utc::now(),
            };
            relations.push(relation);
        }
    }

    Some((entities, relations))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_file() {
        let code = r#"
pub struct ServiceConfig {
    pub port: u16,
}

pub enum LogLevel {
    Info,
    Error,
}

pub trait Runner {
    fn run(&self);
}

pub fn start_server() {
}
"#;
        let result = parse_file(Path::new("src/server.rs"), code);
        assert!(result.is_some());
        let (entities, relations) = result.unwrap();
        assert_eq!(entities.len(), 4);
        assert_eq!(relations.len(), 4);
        assert!(entities.iter().any(|e| e.name == "ServiceConfig" && e.kind == EntityKind::Struct));
        assert!(entities.iter().any(|e| e.name == "LogLevel" && e.kind == EntityKind::Enum));
        assert!(entities.iter().any(|e| e.name == "Runner" && e.kind == EntityKind::Trait));
        assert!(entities.iter().any(|e| e.name == "start_server" && e.kind == EntityKind::Function));
    }

    #[test]
    fn test_parse_python_file() {
        let code = r#"
def compute_sum(a, b):
    return a + b

class MathEngine:
    pass
"#;
        let result = parse_file(Path::new("scripts/engine.py"), code);
        assert!(result.is_some());
        let (entities, relations) = result.unwrap();
        assert_eq!(entities.len(), 2);
        assert_eq!(relations.len(), 2);
        assert!(entities.iter().any(|e| e.name == "compute_sum" && e.kind == EntityKind::Function));
        assert!(entities.iter().any(|e| e.name == "MathEngine" && e.kind == EntityKind::Struct));
    }

    #[test]
    fn test_parse_typescript_file() {
        let code = r#"
export interface UserProfile {
    id: string;
}

export type StringOrNumber = string | number;

export class AuthService {
}

export function fetchUser() {
    return null;
}
"#;
        let result = parse_file(Path::new("web/user.ts"), code);
        assert!(result.is_some());
        let (entities, _) = result.unwrap();
        assert!(entities.iter().any(|e| e.name == "UserProfile" && e.kind == EntityKind::Trait));
        assert!(entities.iter().any(|e| e.name == "StringOrNumber" && e.kind == EntityKind::Struct));
        assert!(entities.iter().any(|e| e.name == "AuthService" && e.kind == EntityKind::Struct));
        assert!(entities.iter().any(|e| e.name == "fetchUser" && e.kind == EntityKind::Function));
    }

    #[test]
    fn test_parse_go_file() {
        let code = r#"
package main

type Config struct {
    Host string
}

func Connect() error {
    return nil
}
"#;
        let result = parse_file(Path::new("server.go"), code);
        assert!(result.is_some());
        let (entities, _) = result.unwrap();
        assert!(entities.iter().any(|e| e.name == "Config" && e.kind == EntityKind::Struct));
        assert!(entities.iter().any(|e| e.name == "Connect" && e.kind == EntityKind::Function));
    }

    #[test]
    fn test_parse_javascript_file() {
        let code = r#"
class Controller {
}

function init() {
}
"#;
        let result = parse_file(Path::new("app.js"), code);
        assert!(result.is_some());
        let (entities, _) = result.unwrap();
        assert!(entities.iter().any(|e| e.name == "Controller" && e.kind == EntityKind::Struct));
        assert!(entities.iter().any(|e| e.name == "init" && e.kind == EntityKind::Function));
    }

    #[test]
    fn test_parse_unsupported_file() {
        let result = parse_file(Path::new("notes.txt"), "hello world");
        assert!(result.is_none());
    }
}


