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
    (class_declaration name: (identifier) @class)
    (method_definition name: (property_identifier) @method)
    (interface_declaration name: (type_identifier) @interface)
    (type_alias_declaration name: (type_identifier) @type)
"#;

const GO_QUERY: &str = r#"
    (function_declaration name: (identifier) @func)
    (method_declaration name: (field_identifier) @method)
    (type_spec name: (type_identifier) @type)
"#;

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

