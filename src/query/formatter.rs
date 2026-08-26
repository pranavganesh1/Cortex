// Result formatting utilities
use super::executor::QueryResult;
use comfy_table::{Table, Cell, Attribute, ContentArrangement};
use crate::core::models::EntityKind;

pub fn format_result(result: &QueryResult) -> String {
    if result.entities.is_empty() && result.total_count == 0 {
        return "No results found.".to_string();
    }

    if result.entities.is_empty() && result.total_count > 0 {
        return format!("Count: {}", result.total_count);
    }

    let mut output = format!("Found {} {}:\n\n", result.total_count, result.query_description);

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("Kind").add_attribute(Attribute::Bold),
        Cell::new("Name").add_attribute(Attribute::Bold),
        Cell::new("Source").add_attribute(Attribute::Bold),
        Cell::new("ID").add_attribute(Attribute::Bold),
    ]);

    for entity in &result.entities {
        let id_short = if entity.id.len() > 40 {
            format!("{}...", &entity.id[..40])
        } else {
            entity.id.clone()
        };

        table.add_row(vec![
            Cell::new(format!("{} {}", entity.kind.icon(), entity.kind)),
            Cell::new(&entity.name),
            Cell::new(&entity.source),
            Cell::new(id_short),
        ]);
    }

    output.push_str(&table.to_string());

    // Show relations if any
    if !result.relations.is_empty() {
        output.push_str(&format!("\n\n{} relations:\n", result.relations.len()));
        let mut rel_table = Table::new();
        rel_table.set_header(vec!["From", "Relation", "To"]);
        for rel in &result.relations {
            let from_short = truncate(&rel.from_id, 30);
            let to_short = truncate(&rel.to_id, 30);
            rel_table.add_row(vec![
                from_short,
                format!("{}", rel.kind),
                to_short,
            ]);
        }
        output.push_str(&rel_table.to_string());
    }

    // Append decision details
    let decisions: Vec<_> = result.entities.iter().filter(|e| matches!(e.kind, EntityKind::Decision)).collect();
    if !decisions.is_empty() {
        output.push_str("\n\n💡 Decisions found:\n");
        for (i, d) in decisions.iter().enumerate() {
            output.push_str(&format!("\n{}. ", i + 1));
            if let Some(content) = &d.content {
                for line in content.lines() {
                    output.push_str(&format!("   {}\n", line));
                }
            }
        }
    }

    output
}

pub fn format_result_json(result: &QueryResult) -> String {
    let json_entities: Vec<serde_json::Value> = result
        .entities
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "name": e.name,
                "kind": e.kind.to_string(),
                "source": e.source,
                "content": e.content,
            })
        })
        .collect();

    let json_relations: Vec<serde_json::Value> = result
        .relations
        .iter()
        .map(|r| {
            serde_json::json!({
                "from_id": r.from_id,
                "to_id": r.to_id,
                "kind": r.kind.to_string(),
            })
        })
        .collect();

    let payload = serde_json::json!({
        "query_description": result.query_description,
        "total_count": result.total_count,
        "entities": json_entities,
        "relations": json_relations,
    });

    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::Entity;

    #[test]
    fn test_format_result_empty() {
        let res = QueryResult {
            entities: vec![],
            relations: vec![],
            total_count: 0,
            query_description: "functions".into(),
        };
        assert_eq!(format_result(&res), "No results found.");
    }

    #[test]
    fn test_format_result_json() {
        let now = chrono::Utc::now();
        let res = QueryResult {
            entities: vec![Entity {
                id: "fn:main".into(),
                name: "main".into(),
                kind: EntityKind::Function,
                source: "src/main.rs".into(),
                content: Some("fn main() {}".into()),
                created_at: now,
                updated_at: now,
                parent_id: None,
            }],
            relations: vec![],
            total_count: 1,
            query_description: "functions".into(),
        };
        let json = format_result_json(&res);
        assert!(json.contains("fn:main"));
        assert!(json.contains("Function"));
        assert!(json.contains("src/main.rs"));
    }
}
