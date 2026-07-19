use super::reconstructor::{MentalState, WorkSession};
#[allow(unused_imports)]
use comfy_table::{Table, Column, Cell, Attribute};

pub fn format_timeline(sessions: &[WorkSession]) -> String {
    if sessions.is_empty() {
        return "No activity recorded in this period.".to_string();
    }

    let mut output = format!("📅 {} work sessions found\n\n", sessions.len());

    for (i, session) in sessions.iter().enumerate() {
        let date = session.start.format("%a %b %d");
        let start = session.start.format("%H:%M");
        let end = session.end.format("%H:%M");
        let dur = if session.duration_minutes < 60 {
            format!("{}m", session.duration_minutes)
        } else {
            format!("{}h {}m", session.duration_minutes / 60, session.duration_minutes % 60)
        };

        output.push_str(&format!("Session {} — {}  {} → {}  ({})\n", i + 1, date, start, end, dur));

        if let Some(active) = &session.active_file {
            output.push_str(&format!("  🎯 Focus: {}\n", active));
        }

        if !session.files_touched.is_empty() {
            output.push_str(&format!("  📄 {} files touched\n", session.files_touched.len()));
        }

        if !session.commits.is_empty() {
            for c in &session.commits {
                let short = if c.len() > 40 { &c[..40] } else { c };
                output.push_str(&format!("  📝 {}\n", short));
            }
        }

        if !session.queries.is_empty() {
            for q in &session.queries {
                let short = if q.len() > 60 { &q[..60] } else { q };
                output.push_str(&format!("  ❓ \"{}\"\n", short));
            }
        }

        if !session.decisions.is_empty() {
            output.push_str(&format!("  💡 {} decisions recorded\n", session.decisions.len()));
        }

        output.push('\n');
    }

    output
}

pub fn format_mental_state(state: &MentalState) -> String {
    let mut output = String::new();

    output.push_str("╔══════════════════════════════════════════════════════╗\n");
    output.push_str("║           🧠 MENTAL STATE RECONSTRUCTION             ║\n");
    output.push_str("╚══════════════════════════════════════════════════════╝\n\n");

    output.push_str(&format!("📍 {}\n", state.timestamp.format("%A, %B %d at %I:%M %p")));

    let s = &state.session;
    let dur = if s.duration_minutes < 60 {
        format!("{}m", s.duration_minutes)
    } else {
        format!("{}h {}m", s.duration_minutes / 60, s.duration_minutes % 60)
    };

    output.push_str(&format!(
        "⏱️  Session: {} → {} ({})\n\n",
        s.start.format("%H:%M"),
        s.end.format("%H:%M"),
        dur
    ));

    if let Some(active) = &state.active_file {
        output.push_str(&format!("🎯 Active file: {}\n", active));
    }

    if !state.files_in_session.is_empty() {
        output.push_str(&format!("📄 Files in this session ({}):\n", state.files_in_session.len()));
        for f in &state.files_in_session {
            let short = if f.len() > 50 { &f[..50] } else { f };
            output.push_str(&format!("   • {}\n", short));
        }
        output.push('\n');
    }

    if !state.recent_commits.is_empty() {
        output.push_str("📝 Recent commits:\n");
        for c in &state.recent_commits {
            let short = if c.len() > 45 { &c[..45] } else { c };
            output.push_str(&format!("   • {}\n", short));
        }
        output.push('\n');
    }

    if !state.recent_decisions.is_empty() {
        output.push_str("💡 Decisions nearby:\n");
        for d in &state.recent_decisions {
            let short = if d.len() > 60 { &d[..60] } else { d };
            output.push_str(&format!("   • {}\n", short));
        }
        output.push('\n');
    }

    if !state.recent_queries.is_empty() {
        output.push_str("❓ Questions you asked:\n");
        for q in &state.recent_queries {
            let short = if q.len() > 60 { &q[..60] } else { q };
            output.push_str(&format!("   • \"{}\"\n", short));
        }
        output.push('\n');
    }

    // Narrative
    output.push_str("─ Context ─\n");
    if let Some(active) = &state.active_file {
        let file_count = state.files_in_session.len();
        if file_count > 1 {
            output.push_str(&format!(
                "You were working on {} files, primarily {}.\n",
                file_count, active
            ));
        } else {
            output.push_str(&format!("You were focused on {}.\n", active));
        }
    }

    if !state.recent_commits.is_empty() {
        output.push_str("You had recently committed code.\n");
    }

    if !state.recent_queries.is_empty() {
        output.push_str("You were actively querying Cortex for context.\n");
    }

    output
}
