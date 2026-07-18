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

/// Confidence scoring
pub fn score_decision(text: &str) -> u8 {
    let mut score = 50u8; // base confidence
    
    // Strong signals
    if text.contains("because") || text.contains("since") || text.contains("due to") {
        score = score.saturating_add(20);
    }
    if text.contains("chose") || text.contains("decided") || text.contains("switched") {
        score = score.saturating_add(15);
    }
    if text.contains("over ") || text.contains("instead of") || text.contains("rather than") {
        score = score.saturating_add(10); // comparative = stronger decision
    }
    
    // Weak signals / penalties
    if text.len() < 20 {
        score = score.saturating_sub(15); // too short = vague
    }
    if text.contains("maybe") || text.contains("perhaps") || text.contains("consider") {
        score = score.saturating_sub(10); // tentative
    }
    if text.contains("TODO") && !text.contains("because") {
        score = score.saturating_sub(15); // TODO without reasoning
    }
    
    score
}
