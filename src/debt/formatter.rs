use super::detector::{DebtItem, DebtKind, Severity};

pub fn format_debt_report(debts: &[DebtItem]) -> String {
    if debts.is_empty() {
        return "🎉 No cognitive debt detected. Your mind is clear.\n".to_string();
    }

    let critical = debts.iter().filter(|d| d.severity == Severity::Critical).count();
    let high = debts.iter().filter(|d| d.severity == Severity::High).count();
    let medium = debts.iter().filter(|d| d.severity == Severity::Medium).count();
    let low = debts.iter().filter(|d| d.severity == Severity::Low).count();
    
    let mut output = String::new();
    output.push_str("╔══════════════════════════════════════════════════════╗\n");
    output.push_str("║         🧠 COGNITIVE DEBT DASHBOARD                  ║\n");
    output.push_str("╚══════════════════════════════════════════════════════╝\n\n");
    
    output.push_str(&format!("📊 {} critical  {} high  {} medium  {} low\n\n", critical, high, medium, low));

    let groups = [
        (DebtKind::OpenRefactor, "🔥 Open Refactors"),
        (DebtKind::StaleDecision, "💡 Stale Decisions"),
        (DebtKind::AbandonedSession, "📁 Abandoned Work"),
        (DebtKind::TodoWithoutAction, "⏳ Old TODOs"),
        (DebtKind::UnresolvedQuery, "❓ Unresolved Queries"),
    ];
    
    for (kind, label) in groups {
        let items: Vec<_> = debts.iter().filter(|d| d.kind == kind).collect();
        if items.is_empty() { continue; }
        
        output.push_str(&format!("{}\n{}\n", label, "─".repeat(50)));
        
        for item in items {
            let emoji = match item.severity {
                Severity::Critical => "🔴",
                Severity::High => "🟠",
                Severity::Medium => "🟡",
                Severity::Low => "🟢",
            };
            let days = (chrono::Utc::now() - item.timestamp).num_days();
            let ago = if days == 0 { "today".into() } else { format!("{}d ago", days) };
            
            output.push_str(&format!("  {} {}  ({})\n", emoji, item.title, ago));
            output.push_str(&format!("     {}\n\n", item.description));
        }
    }
    
    output.push_str("💬 Tip: `cortex back` resumes an abandoned context.\n");
    output
}

pub fn format_weekly_report(debts: &[DebtItem]) -> String {
    let mut out = format_debt_report(debts);
    
    out.push_str("\n╔══════════════════════════════════════════════════════╗\n");
    out.push_str("║              📅 THIS WEEK'S PATTERNS                 ║\n");
    out.push_str("╚══════════════════════════════════════════════════════╝\n\n");
    
    let abandoned = debts.iter().filter(|d| d.kind == DebtKind::AbandonedSession).count();
    let unresolved = debts.iter().filter(|d| d.kind == DebtKind::UnresolvedQuery).count();
    let stale = debts.iter().filter(|d| d.kind == DebtKind::StaleDecision).count();
    let todos = debts.iter().filter(|d| d.kind == DebtKind::TodoWithoutAction).count();
    
    if abandoned > 0 {
        out.push_str(&format!("→ You abandoned {} work sessions without committing.\n", abandoned));
        out.push_str("  Fix: Set a timer. Commit every 25 minutes.\n\n");
    }
    
    if unresolved > 0 {
        out.push_str(&format!("→ You asked {} questions but didn't follow up with code.\n", unresolved));
        out.push_str("  Fix: Ask → Code → Commit. Don't just ask.\n\n");
    }
    
    if stale > 0 {
        out.push_str(&format!("→ You made {} decisions that still need implementation.\n", stale));
        out.push_str("  Fix: Block 2 hours this week to close decision loops.\n\n");
    }
    
    if todos > 0 {
        out.push_str(&format!("→ You have {} TODOs rotting in code.\n", todos));
        out.push_str("  Fix: Either do them or delete them.\n\n");
    }
    
    if abandoned == 0 && unresolved == 0 && stale == 0 && todos == 0 {
        out.push_str("→ Clean week. No patterns to fix.\n");
    }
    
    out
}
