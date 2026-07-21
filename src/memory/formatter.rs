use super::context::ContextFrame;
use super::stack::with_stack;
use comfy_table::{Table, Cell, Attribute};

pub fn format_focus() -> String {
    let mut output = String::new();
    
    with_stack(|wm| {
        let current = wm.current();
        
        output.push_str("╔══════════════════════════════════════════════════════╗\n");
        output.push_str("║              🎯 CURRENT FOCUS                        ║\n");
        output.push_str("╚══════════════════════════════════════════════════════╝\n\n");
        
        output.push_str(&format!("📁 Topic: {}\n", current.topic));
        output.push_str(&format!("⏱️  Active for: {} min\n", current.duration_minutes()));
        output.push_str(&format!("🕐 Last active: {}\n", current.last_active.format("%H:%M:%S")));
        output.push_str("\n");
        
        if !current.files.is_empty() {
            output.push_str(&format!("📄 Files ({}):\n", current.files.len()));
            for f in &current.files {
                output.push_str(&format!("   • {}\n", f));
            }
            output.push('\n');
        }
        
        if !current.decisions.is_empty() {
            output.push_str(&format!("💡 Decisions ({}):\n", current.decisions.len()));
            for d in &current.decisions {
                let short = if d.len() > 70 { &d[..70] } else { d };
                output.push_str(&format!("   • {}\n", short));
            }
            output.push('\n');
        }
        
        if !current.queries.is_empty() {
            output.push_str(&format!("❓ Queries ({}):\n", current.queries.len()));
            for q in &current.queries {
                let short = if q.len() > 70 { &q[..70] } else { q };
                output.push_str(&format!("   • \"{}\"\n", short));
            }
            output.push('\n');
        }
        
        if !current.commits.is_empty() {
            output.push_str(&format!("📝 Commits ({}):\n", current.commits.len()));
            for c in &current.commits[..current.commits.len().min(5)] {
                let short = if c.len() > 40 { &c[..40] } else { c };
                output.push_str(&format!("   • {}\n", short));
            }
        }
    });
    
    output
}

pub fn format_stack() -> String {
    let mut output = String::new();
    
    with_stack(|wm| {
        let contexts = wm.all_contexts();
        
        output.push_str("╔══════════════════════════════════════════════════════╗\n");
        output.push_str("║              🧠 WORKING MEMORY STACK                 ║\n");
        output.push_str("╚══════════════════════════════════════════════════════╝\n\n");
        
        if contexts.len() <= 1 {
            output.push_str("Only one active context. Start working on something else to build the stack.\n");
            return;
        }
        
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("").add_attribute(Attribute::Bold),
            Cell::new("Topic").add_attribute(Attribute::Bold),
            Cell::new("Duration").add_attribute(Attribute::Bold),
            Cell::new("Files").add_attribute(Attribute::Bold),
            Cell::new("Last Active").add_attribute(Attribute::Bold),
        ]);
        
        for (_i, ctx) in contexts.iter().enumerate() {
            let marker = if ctx.is_active { "▶" } else { " " };
            let dur = if ctx.duration_minutes() < 60 {
                format!("{}m", ctx.duration_minutes())
            } else {
                format!("{}h {}m", ctx.duration_minutes() / 60, ctx.duration_minutes() % 60)
            };
            
            table.add_row(vec![
                marker.to_string(),
                ctx.topic.clone(),
                dur,
                ctx.files.len().to_string(),
                ctx.last_active.format("%H:%M").to_string(),
            ]);
        }
        
        output.push_str(&table.to_string());
        output.push_str("\n\n▶ = current context. Use `cortex back` to restore previous.");
    });
    
    output
}

pub fn format_where_was_i() -> String {
    let mut output = String::new();
    
    with_stack(|wm| {
        let recent = wm.recent_contexts(3);
        
        output.push_str("╔══════════════════════════════════════════════════════╗\n");
        output.push_str("║              🤔 WHERE WAS I?                         ║\n");
        output.push_str("╚══════════════════════════════════════════════════════╝\n\n");
        
        if recent.len() <= 1 {
            output.push_str("You've been focused on one thing. Keep going.\n");
            return;
        }
        
        // Current context
        let current = &recent[0];
        output.push_str(&format!("🎯 Currently: {} ({} min)\n", current.topic, current.duration_minutes()));
        output.push_str(&format!("   Files: {}\n", current.files.join(", ")));
        output.push('\n');
        
        // Previous context
        if let Some(prev) = recent.get(1) {
            let gap = (current.start_time - prev.last_active).num_minutes();
            output.push_str(&format!("⏮️  Before that: {} ({} min, {} min ago)\n", 
                prev.topic, 
                prev.duration_minutes(),
                gap
            ));
            if !prev.files.is_empty() {
                output.push_str(&format!("   Files: {}\n", prev.files.join(", ")));
            }
            if !prev.decisions.is_empty() {
                let last = &prev.decisions[prev.decisions.len() - 1];
                let short = if last.len() > 60 { &last[..60] } else { last };
                output.push_str(&format!("   Last decision: {}\n", short));
            }
            if !prev.queries.is_empty() {
                let last = &prev.queries[prev.queries.len() - 1];
                let short = if last.len() > 60 { &last[..60] } else { last };
                output.push_str(&format!("   Last question: \"{}\"\n", short));
            }
        }
        
        // Deeper history
        if recent.len() > 2 {
            output.push_str("\n📜 Earlier today:\n");
            for ctx in &recent[2..] {
                output.push_str(&format!("   • {} ({} min)\n", ctx.topic, ctx.duration_minutes()));
            }
        }
    });
    
    output
}

pub fn format_back_result(prev: &ContextFrame) -> String {
    let mut output = String::new();
    
    output.push_str("╔══════════════════════════════════════════════════════╗\n");
    output.push_str("║              ⏮️  CONTEXT RESTORED                    ║\n");
    output.push_str("╚══════════════════════════════════════════════════════╝\n\n");
    
    output.push_str(&format!("📁 Restored: {}\n", prev.topic));
    output.push_str(&format!("⏱️  You worked on this for {} min\n", prev.duration_minutes()));
    output.push_str(&format!("🕐 Last touched: {}\n\n", prev.last_active.format("%H:%M:%S")));
    
    if !prev.files.is_empty() {
        output.push_str("📄 Files in this context:\n");
        for f in &prev.files {
            output.push_str(&format!("   • {}\n", f));
        }
    }
    
    if !prev.decisions.is_empty() {
        output.push_str("\n💡 Decisions you made:\n");
        for d in &prev.decisions {
            let short = if d.len() > 70 { &d[..70] } else { d };
            output.push_str(&format!("   • {}\n", short));
        }
    }
    
    output.push_str("\n💬 Tip: Open these files in your editor to resume where you left off.\n");
    
    output
}
