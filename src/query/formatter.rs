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
        let kind_emoji = match entity.kind {
            EntityKind::Function => "f",
            EntityKind::Struct => "S",
            EntityKind::Class => "C",
            EntityKind::Enum => "E",
            EntityKind::Trait => "T",
            EntityKind::Interface => "I",
            EntityKind::File => "F",
            EntityKind::Commit => "G",
            EntityKind::Module => "M",
            EntityKind::Decision => "D",
            EntityKind::Note => "N",
        };

        let id_short = if entity.id.len() > 40 {
            format!("{}...", &entity.id[..40])
        } else {
            entity.id.clone()
        };

        table.add_row(vec![
            Cell::new(format!("{} {:?}", kind_emoji, entity.kind)),
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
                format!("{:?}", rel.kind),
                to_short,
            ]);
        }
        output.push_str(&rel_table.to_string());
    }

    output
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}
