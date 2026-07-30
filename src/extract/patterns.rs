use regex::Regex;
use once_cell::sync::Lazy;

/// Decision patterns with capture groups:
/// $1 = choice made, $2 = reason (optional)
pub static DECISION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // "We chose X because Y"
        Regex::new(r"(?i)(?:chose|chosen|choose|picked|selected|went with|settled on)\s+(\w+(?:\s+\w+){0,5})\s+(?:because|since|as|due to|for)\s+(.{3,200})").unwrap(),
        
        // "Decided to use X over Y because Z"
        Regex::new(r"(?i)(?:decided|decision)\s+(?:to\s+)?(?:use|go with|adopt|migrate to|switch to)\s+(\w+(?:\s+\w+){0,5})\s+(?:over|instead of|rather than|vs\.?)\s+(\w+(?:\s+\w+){0,5})(?:\s+(?:because|since|as|due to|for)\s+(.{3,200}))?").unwrap(),
        
        // "Switched from X to Y due to Z"
        Regex::new(r"(?i)(?:switched|migrated|moved|transitioned|converted)\s+(?:from\s+)?(\w+(?:\s+\w+){0,5})\s+to\s+(\w+(?:\s+\w+){0,5})(?:\s+(?:due to|because of|thanks to|owing to)\s+(.{3,200}))?").unwrap(),
        
        // "Using X instead of Y because Z"
        Regex::new(r"(?i)(?:using|use|went with)\s+(\w+(?:\s+\w+){0,5})\s+(?:instead of|rather than|over|not)\s+(\w+(?:\s+\w+){0,5})(?:\s+(?:because|since|as)\s+(.{3,200}))?").unwrap(),
        
        // "Replaced X with Y for Z"
        Regex::new(r"(?i)(?:replaced?|substituted?|swapped)\s+(\w+(?:\s+\w+){0,5})\s+(?:with|by|for)\s+(\w+(?:\s+\w+){0,5})(?:\s+(?:to|for|so|in order to)\s+(.{3,200}))?").unwrap(),
        
        // "Don't use X — causes Y" / "Avoid X because Y"
        Regex::new(r"(?i)(?:don't|do not|avoid|never|stop)\s+(?:use|using)\s+(\w+(?:\s+\w+){0,5})(?:\s*[—–-]\s*|\s+because\s+|\s*:?\s*)(.{3,200})").unwrap(),
        
        // "Added X to solve Y"
        Regex::new(r"(?i)(?:added|introduced|brought in|implemented)\s+(\w+(?:\s+\w+){0,5})\s+(?:to|in order to|so we can|for)\s+(.{3,200})").unwrap(),
        
        // "Removed X because Y"
        Regex::new(r"(?i)(?:removed|deleted|dropped|got rid of|eliminated)\s+(\w+(?:\s+\w+){0,5})(?:\s+(?:because|since|as|due to|after)\s+(.{3,200}))?").unwrap(),
        
        // "Use X for Y" (weaker signal, needs context)
        Regex::new(r"(?i)(?:we\s+)?(?:use|using|utilize)\s+(\w+(?:\s+\w+){0,5})\s+(?:for|to|in order to)\s+(.{3,200})").unwrap(),
    ]
});

/// Code comment decision markers
pub static COMMENT_DECISION_MARKERS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)(?:TODO|NOTE|HACK|FIXME|DECISION|REVIEW|OPTIMIZE|WARN):\s*(.{5,300})").unwrap(),
        Regex::new(r"(?i)(?:we|i)\s+(?:chose|picked|decided|went with)\s+(.{5,200})").unwrap(),
        Regex::new(r"(?i)(?:this\s+is|we're|we are)\s+(?:using|running on|built with)\s+(.{5,200})").unwrap(),
    ]
});

/// Confidence scoring for decision text.
///
/// Returns a score from 0-100 indicating how confident we are that the
/// given text represents an actual architectural/engineering decision.
/// Higher scores mean stronger decision signals.
pub fn score_decision(text: &str) -> u8 {
    let lower = text.to_lowercase();
    let mut score = 50u8; // base confidence
    
    // === Strong positive signals ===
    
    // Reasoning keywords — a decision with rationale is much stronger
    if lower.contains("because") || lower.contains("since") || lower.contains("due to") {
        score = score.saturating_add(20);
    }
    
    // Action verbs — explicit decision language
    if lower.contains("chose") || lower.contains("decided") || lower.contains("switched") {
        score = score.saturating_add(15);
    }
    
    // Comparisons — choosing one thing over another is a strong signal
    if lower.contains("over ") || lower.contains("instead of") || lower.contains("rather than") {
        score = score.saturating_add(10);
    }
    
    // Architectural concern keywords — decisions about core qualities
    if lower.contains("performance") || lower.contains("security") || lower.contains("scalability")
        || lower.contains("maintainability") || lower.contains("reliability") {
        score = score.saturating_add(10);
    }
    
    // Team consensus — "we agreed", "team decided"
    if lower.contains("we agreed") || lower.contains("team decided") || lower.contains("consensus") {
        score = score.saturating_add(10);
    }
    
    // === Weak signals / penalties ===
    
    // Too short — likely incomplete or vague
    if text.len() < 20 {
        score = score.saturating_sub(15);
    }
    
    // Tentative language — not a firm decision
    if lower.contains("maybe") || lower.contains("perhaps") || lower.contains("consider") {
        score = score.saturating_sub(10);
    }
    
    // TODO without reasoning — a reminder, not a decision
    if lower.contains("todo") && !lower.contains("because") {
        score = score.saturating_sub(15);
    }
    
    // Questions — asking, not deciding
    if text.contains('?') {
        score = score.saturating_sub(10);
    }
    
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strong_decision_with_reasoning() {
        let score = score_decision("We chose Redis over Memcached because it supports persistence");
        // base(50) + because(20) + chose(15) + over(10) = 95
        assert!(score >= 90, "Strong decision should score high, got {}", score);
    }

    #[test]
    fn test_weak_decision_no_reasoning() {
        let score = score_decision("Use serde");
        // base(50) - short(15) = 35
        assert!(score < 50, "Vague short note should score low, got {}", score);
    }

    #[test]
    fn test_tentative_language_penalized() {
        let score = score_decision("Maybe we should consider using a different database");
        // base(50) - maybe(10) - consider(10) = 30 (both match)
        // Actually "consider" is checked separately so -10, and "maybe" is separate -10
        assert!(score < 50, "Tentative language should reduce score, got {}", score);
    }

    #[test]
    fn test_todo_without_reason_penalized() {
        let score = score_decision("TODO: refactor this module later");
        // base(50) - todo_no_because(15) = 35
        assert!(score < 50, "TODO without reason should score low, got {}", score);
    }

    #[test]
    fn test_todo_with_reason_not_penalized() {
        let score = score_decision("TODO: refactor this module because the current approach is fragile");
        // base(50) + because(20) = 70 (todo penalty NOT applied since 'because' is present)
        assert!(score >= 65, "TODO with reasoning should score decent, got {}", score);
    }

    #[test]
    fn test_architectural_concern_boosts_score() {
        let score = score_decision("Switched to async runtime because of performance requirements");
        // base(50) + because(20) + switched(15) + performance(10) = 95
        assert!(score >= 90, "Architectural decision should score very high, got {}", score);
    }

    #[test]
    fn test_question_penalized() {
        let score = score_decision("Should we use Redis instead of Postgres?");
        // base(50) + instead_of(10) - question(10) = 50
        let score_without_question = score_decision("We should use Redis instead of Postgres");
        assert!(score < score_without_question, "Question should score lower than statement");
    }

    #[test]
    fn test_team_consensus_boosts_score() {
        let score = score_decision("We agreed to use TypeScript for the frontend because of type safety");
        // base(50) + because(20) + we_agreed(10) = 80
        assert!(score >= 75, "Team consensus should boost score, got {}", score);
    }

    #[test]
    fn test_decision_patterns_match_common_phrases() {
        let test_cases = vec![
            "We chose Rust because of memory safety",
            "Decided to use PostgreSQL over MySQL",
            "Switched from REST to GraphQL due to performance",
            "Using Redis instead of Memcached",
            "Added caching to improve response times",
        ];

        for text in test_cases {
            let matched = DECISION_PATTERNS.iter().any(|re| re.is_match(text));
            assert!(matched, "Pattern should match: '{}'", text);
        }
    }

    #[test]
    fn test_comment_markers_match() {
        let test_cases = vec![
            "TODO: migrate to the new API before release",
            "DECISION: use connection pooling for database access",
            "NOTE: this workaround is needed until upstream fixes the bug",
        ];

        for text in test_cases {
            let matched = COMMENT_DECISION_MARKERS.iter().any(|re| re.is_match(text));
            assert!(matched, "Comment marker should match: '{}'", text);
        }
    }
}
