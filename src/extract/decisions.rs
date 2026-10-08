use crate::core::models::{Entity, EntityKind, Relation, RelationKind};
use crate::db::{insert_entities, insert_relations};
use sqlx::SqlitePool;
use chrono::Utc;
use anyhow::Result;
use super::patterns::{DECISION_PATTERNS, COMMENT_DECISION_MARKERS, score_decision};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDecision {
    pub text: String,
    pub choice: String,        // what was chosen
    pub alternative: Option<String>, // what was rejected (if any)
    pub reason: Option<String>,    // why
    pub confidence: u8,
    pub source: String,        // commit hash, file path, etc.
    pub source_kind: SourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    CommitMessage,
    CodeComment,
    PrDescription,
    Manual,
}

pub fn extract_from_text(text: &str, source: &str, source_kind: SourceKind) -> Vec<ExtractedDecision> {
    let mut decisions = Vec::new();
    
    for pattern in DECISION_PATTERNS.iter() {
        for cap in pattern.captures_iter(text) {
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let choice = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("").to_string();
            let alternative = cap.get(2).map(|m| Some(m.as_str().trim().to_string())).unwrap_or(None);
            let reason = cap.get(3).map(|m| Some(m.as_str().trim().to_string())).unwrap_or(None);
            
            // If no explicit reason capture, try to extract from full match
            let reason = reason.or_else(|| extract_reason(full_match));
            
            let confidence = score_decision(full_match);
            
            if !choice.is_empty() && confidence >= 40 {
                decisions.push(ExtractedDecision {
                    text: full_match.to_string(),
                    choice,
                    alternative,
                    reason,
                    confidence,
                    source: source.to_string(),
                    source_kind: source_kind.clone(),
                });
            }
        }
    }
    
    decisions
}

pub fn extract_from_comments(text: &str, file_path: &str) -> Vec<ExtractedDecision> {
    let mut decisions = Vec::new();
    
    for line in text.lines() {
        let trimmed = line.trim();
        // Only look at comment lines
        if !trimmed.starts_with("//") && !trimmed.starts_with("#") && !trimmed.starts_with("/*") && !trimmed.starts_with("*") {
            continue;
        }
        
        for pattern in COMMENT_DECISION_MARKERS.iter() {
            if let Some(cap) = pattern.captures(trimmed) {
                let full = cap.get(0).map(|m| m.as_str()).unwrap_or("");
                let content = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                
                // Try to parse the comment content as a decision
                let inner = extract_from_text(content, file_path, SourceKind::CodeComment);
                decisions.extend(inner);
                
                // Also store the raw comment if it looks decision-like
                if score_decision(content) >= 45 {
                    decisions.push(ExtractedDecision {
                        text: full.to_string(),
                        choice: content.to_string(),
                        alternative: None,
                        reason: extract_reason(content),
                        confidence: score_decision(content).saturating_sub(10),
                        source: file_path.to_string(),
                        source_kind: SourceKind::CodeComment,
                    });
                }
            }
        }
    }
    
    decisions
}

fn extract_reason(text: &str) -> Option<String> {
    // Simple reason extraction: look for "because", "since", "due to", "for", "to"
    let markers = ["because ", "since ", "due to ", "as ", "for ", "in order to ", "to "];
    for marker in markers {
        if let Some(pos) = text.to_lowercase().find(marker) {
            let start = pos + marker.len();
            let reason = text[start..].trim();
            if reason.len() > 5 {
                return Some(reason.to_string());
            }
        }
    }
    None
}

/// Convert extracted decisions to graph entities and store them
pub async fn store_decisions(
    pool: &SqlitePool,
    decisions: &[ExtractedDecision],
    parent_id: Option<String>,
) -> Result<()> {
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    
    for d in decisions {
        let decision_id = format!(
            "decision:{}:{}",
            sanitize(&d.choice),
            &d.source.replace('/', "-")
        );
        
        let content = format!(
            "Decision: {}\nAlternative: {}\nReason: {}\nSource: {:?} ({})\nConfidence: {}/100",
            d.choice,
            d.alternative.as_deref().unwrap_or("none"),
            d.reason.as_deref().unwrap_or("unspecified"),
            d.source_kind,
            d.source,
            d.confidence
        );
        
        entities.push(Entity {
            id: decision_id.clone(),
            kind: EntityKind::Decision,
            name: format!("{} → {}", d.choice, d.reason.as_deref().unwrap_or("?")),
            content: Some(content),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: format!("extract:{:?}", d.source_kind),
            parent_id: parent_id.clone(),
        });
        
        // Log decision event
        let _ = crate::core::events::log_event(pool, &decision_id, "decision:extracted", Some(&d.text)).await;
        
        // Push to working memory
        crate::memory::stack::add_decision(&d.text);
        
        // Link to parent if provided
        if let Some(pid) = &parent_id {
            relations.push(Relation {
                id: format!("rel:decided_in:{}:{}", pid, decision_id),
                from_id: pid.clone(),
                to_id: decision_id.clone(),
                kind: RelationKind::DecidedIn,
                created_at: Utc::now(),
            });
        }
    }
    
    if !entities.is_empty() {
        insert_entities(pool, &entities).await?;
        insert_relations(pool, &relations).await?;
    }
    
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .take(40)
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize("use sqlite"), "use_sqlite");
        assert_eq!(sanitize("Stripe-vs-PayPal!!"), "stripe_vs_paypal__");
        let long = "a".repeat(50);
        assert_eq!(sanitize(&long).len(), 40);
    }

    #[test]
    fn test_extract_reason() {
        assert_eq!(
            extract_reason("chose redis because it is fast and simple"),
            Some("it is fast and simple".to_string())
        );
        assert_eq!(
            extract_reason("switched to tokio due to async ecosystem"),
            Some("async ecosystem".to_string())
        );
        assert_eq!(extract_reason("no reason specified"), None);
    }

    #[test]
    fn test_extract_from_text_valid_decision() {
        let text = "We chose PostgreSQL because of jsonb support";
        let decisions = extract_from_text(text, "commit_1", SourceKind::CommitMessage);
        assert!(!decisions.is_empty());
        let d = &decisions[0];
        assert_eq!(d.choice, "PostgreSQL");
        assert_eq!(d.source, "commit_1");
        assert_eq!(d.source_kind, SourceKind::CommitMessage);
        assert!(d.confidence >= 40);
    }

    #[test]
    fn test_extract_from_comments() {
        let comment_text = "// DECISION: choose axum for api server\n// regular comment here\n";
        let decisions = extract_from_comments(comment_text, "src/server.rs");
        assert!(!decisions.is_empty());
        assert_eq!(decisions[0].source_kind, SourceKind::CodeComment);
        assert_eq!(decisions[0].source, "src/server.rs");
    }
}
