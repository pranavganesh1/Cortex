use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebtKind {
    AbandonedSession,
    UnresolvedQuery,
    StaleDecision,
    OpenRefactor,
    TodoWithoutAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDebtKindError(pub String);

impl std::fmt::Display for ParseDebtKindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown debt kind: {}", self.0)
    }
}

impl std::error::Error for ParseDebtKindError {}

impl std::str::FromStr for DebtKind {
    type Err = ParseDebtKindError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        match normalized.as_str() {
            "abandoned_session" | "abandoned" => Ok(DebtKind::AbandonedSession),
            "unresolved_query" | "unresolved" | "query" => Ok(DebtKind::UnresolvedQuery),
            "stale_decision" | "stale" | "decision" => Ok(DebtKind::StaleDecision),
            "open_refactor" | "refactor" => Ok(DebtKind::OpenRefactor),
            "todo_without_action" | "todo" => Ok(DebtKind::TodoWithoutAction),
            _ => Err(ParseDebtKindError(s.to_string())),
        }
    }
}

impl std::fmt::Display for DebtKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebtKind::AbandonedSession => write!(f, "Abandoned Session"),
            DebtKind::UnresolvedQuery => write!(f, "Unresolved Query"),
            DebtKind::StaleDecision => write!(f, "Stale Decision"),
            DebtKind::OpenRefactor => write!(f, "Open Refactor"),
            DebtKind::TodoWithoutAction => write!(f, "TODO Without Action"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseSeverityError(pub String);

impl std::fmt::Display for ParseSeverityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown severity level: {}", self.0)
    }
}

impl std::error::Error for ParseSeverityError {}

impl std::str::FromStr for Severity {
    type Err = ParseSeverityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "low" => Ok(Severity::Low),
            "medium" | "med" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" | "crit" => Ok(Severity::Critical),
            _ => Err(ParseSeverityError(s.to_string())),
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

impl Severity {
    pub fn rank(&self) -> u8 {
        match self {
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        }
    }

    /// Returns the emoji indicator for this severity level
    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
        }
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebtItem {
    pub kind: DebtKind,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub timestamp: DateTime<Utc>,
    pub entity_id: Option<String>,
}

pub fn filter_debt_by_min_severity(debts: &[DebtItem], min_severity: Severity) -> Vec<DebtItem> {
    debts.iter()
        .filter(|d| d.severity >= min_severity)
        .cloned()
        .collect()
}

pub fn filter_debt_by_kind(debts: &[DebtItem], kind: &DebtKind) -> Vec<DebtItem> {
    debts.iter()
        .filter(|d| d.kind == *kind)
        .cloned()
        .collect()
}

pub async fn detect_debt(pool: &SqlitePool) -> Result<Vec<DebtItem>> {
    let mut debts = Vec::new();
    debts.extend(detect_abandoned_sessions(pool).await?);
    debts.extend(detect_unresolved_queries(pool).await?);
    debts.extend(detect_stale_decisions(pool).await?);
    debts.extend(detect_open_refactors(pool).await?);
    debts.extend(detect_old_todos(pool).await?);
    
    // Sort: Critical first, then by recency
    debts.sort_by(|a, b| {
        b.severity.cmp(&a.severity)
            .then_with(|| b.timestamp.cmp(&a.timestamp))
    });
    
    Ok(debts)
}

async fn detect_abandoned_sessions(pool: &SqlitePool) -> Result<Vec<DebtItem>> {
    let mut debts = Vec::new();
    let cutoff = (Utc::now() - Duration::days(3)).to_rfc3339();
    
    // Files saved 3+ days ago, no commit within 24h, and not touched since
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT entity_id, timestamp, metadata 
        FROM events e1
        WHERE action = 'file:save'
        AND timestamp < ?
        AND NOT EXISTS (
            SELECT 1 FROM events e2 
            WHERE e2.action = 'git:commit' 
            AND e2.timestamp > e1.timestamp 
            AND e2.timestamp < datetime(e1.timestamp, '+1 day')
        )
        AND NOT EXISTS (
            SELECT 1 FROM events e3 
            WHERE e3.action = 'file:save' 
            AND e3.entity_id = e1.entity_id
            AND e3.timestamp > datetime('now', '-7 days')
        )
        ORDER BY timestamp DESC
        LIMIT 20
        "#
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await?;
    
    let mut seen = std::collections::HashSet::new();
    for (entity_id, timestamp, metadata) in rows {
        if !seen.insert(entity_id.clone()) { continue; }
        
        let ts = DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc);
        let file = metadata.unwrap_or_else(|| entity_id.clone());
        let file = if let Some(stripped) = file.strip_prefix("file:") { stripped.to_string() } else { file };
        
        debts.push(DebtItem {
            kind: DebtKind::AbandonedSession,
            title: format!("Abandoned: {}", file),
            description: "Worked on this 3+ days ago. No commit. Not touched in a week.".to_string(),
            severity: Severity::Medium,
            timestamp: ts,
            entity_id: Some(entity_id),
        });
    }
    Ok(debts)
}

async fn detect_unresolved_queries(pool: &SqlitePool) -> Result<Vec<DebtItem>> {
    let mut debts = Vec::new();
    let cutoff = (Utc::now() - Duration::days(7)).to_rfc3339();
    
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT id, timestamp, metadata 
        FROM events
        WHERE action = 'query:asked'
        AND timestamp > ?
        AND NOT EXISTS (
            SELECT 1 FROM events e2
            WHERE e2.action IN ('file:save', 'git:commit')
            AND e2.timestamp > events.timestamp
            AND e2.timestamp < datetime(events.timestamp, '+30 minutes')
        )
        ORDER BY timestamp DESC
        LIMIT 20
        "#
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await?;
    
    for (id, timestamp, metadata) in rows {
        let ts = DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc);
        let query = metadata.unwrap_or_else(|| "unknown".to_string());
        if query.len() < 10 { continue; }
        
        debts.push(DebtItem {
            kind: DebtKind::UnresolvedQuery,
            title: format!("Unresolved: \"{}\"", query.chars().take(50).collect::<String>()),
            description: "Asked but no code changes within 30 min.".to_string(),
            severity: Severity::Low,
            timestamp: ts,
            entity_id: Some(id),
        });
    }
    Ok(debts)
}

async fn detect_stale_decisions(pool: &SqlitePool) -> Result<Vec<DebtItem>> {
    let mut debts = Vec::new();
    let cutoff = (Utc::now() - Duration::days(7)).to_rfc3339();
    
    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        r#"
        SELECT id, name, content, created_at
        FROM entities
        WHERE kind = 'Decision'
        AND created_at < ?
        AND created_at > datetime('now', '-30 days')
        AND (
            content LIKE '%will%' OR content LIKE '%plan%' 
            OR content LIKE '%need to%' OR content LIKE '%should%'
            OR content LIKE '%todo%' OR content LIKE '%fix%'
            OR content LIKE '%migrate%' OR content LIKE '%refactor%'
        )
        ORDER BY created_at DESC
        LIMIT 20
        "#
    )
    .bind(&cutoff)
    .fetch_all(pool)
    .await?;
    
    for (id, name, content, created_at) in rows {
        let ts = DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc);
        let preview = content.as_deref().unwrap_or(&name)
            .lines().next().unwrap_or("")
            .chars().take(80).collect::<String>();
        
        debts.push(DebtItem {
            kind: DebtKind::StaleDecision,
            title: format!("Stale decision: {}", name.chars().take(40).collect::<String>()),
            description: format!("Decided '{}' — no follow-up in 7+ days.", preview),
            severity: Severity::High,
            timestamp: ts,
            entity_id: Some(id),
        });
    }
    Ok(debts)
}

async fn detect_open_refactors(pool: &SqlitePool) -> Result<Vec<DebtItem>> {
    let mut debts = Vec::new();
    
    let rows: Vec<(String, i64, String)> = sqlx::query_as(
        r#"
        SELECT entity_id, COUNT(*) as save_count, date(timestamp) as day
        FROM events
        WHERE action = 'file:save'
        AND timestamp > datetime('now', '-14 days')
        GROUP BY entity_id, date(timestamp)
        HAVING save_count > 5
        AND NOT EXISTS (
            SELECT 1 FROM events e2
            WHERE e2.action = 'git:commit'
            AND date(e2.timestamp) = day
        )
        ORDER BY save_count DESC
        LIMIT 10
        "#
    )
    .fetch_all(pool)
    .await?;
    
    for (entity_id, count, day) in rows {
        let file = if let Some(stripped) = entity_id.strip_prefix("file:") { stripped.to_string() } else { entity_id.clone() };
        debts.push(DebtItem {
            kind: DebtKind::OpenRefactor,
            title: format!("Open refactor: {}", file),
            description: format!("{} edits on {} with no commit. You might be mid-refactor.", count, day),
            severity: if count > 10 { Severity::Critical } else { Severity::High },
            timestamp: Utc::now(),
            entity_id: Some(entity_id),
        });
    }
    Ok(debts)
}

async fn detect_old_todos(pool: &SqlitePool) -> Result<Vec<DebtItem>> {
    let mut debts = Vec::new();
    
    let rows: Vec<(String, String, Option<String>, String)> = sqlx::query_as(
        r#"
        SELECT id, name, content, created_at
        FROM entities
        WHERE source = 'extract:CodeComment'
        AND created_at < datetime('now', '-14 days')
        AND created_at > datetime('now', '-90 days')
        AND (
            content LIKE '%TODO%' OR content LIKE '%FIXME%' 
            OR content LIKE '%HACK%' OR content LIKE '%XXX%'
        )
        ORDER BY created_at DESC
        LIMIT 15
        "#
    )
    .fetch_all(pool)
    .await?;
    
    for (id, name, content, created_at) in rows {
        let ts = DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc);
        let days_old = (Utc::now() - ts).num_days();
        let preview = content.as_deref().unwrap_or(&name)
            .lines().next().unwrap_or("")
            .chars().take(100).collect::<String>();
        
        debts.push(DebtItem {
            kind: DebtKind::TodoWithoutAction,
            title: format!("Old TODO ({}d): {}", days_old, name.chars().take(30).collect::<String>()),
            description: preview,
            severity: if days_old > 30 { Severity::High } else { Severity::Medium },
            timestamp: ts,
            entity_id: Some(id),
        });
    }
    Ok(debts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debt_kind_display() {
        assert_eq!(DebtKind::AbandonedSession.to_string(), "Abandoned Session");
        assert_eq!(DebtKind::UnresolvedQuery.to_string(), "Unresolved Query");
        assert_eq!(DebtKind::StaleDecision.to_string(), "Stale Decision");
        assert_eq!(DebtKind::OpenRefactor.to_string(), "Open Refactor");
        assert_eq!(DebtKind::TodoWithoutAction.to_string(), "TODO Without Action");
    }

    #[test]
    fn test_severity_display_and_emoji() {
        assert_eq!(Severity::Critical.to_string(), "Critical");
        assert_eq!(Severity::Critical.emoji(), "🔴");

        assert_eq!(Severity::High.to_string(), "High");
        assert_eq!(Severity::High.emoji(), "🟠");

        assert_eq!(Severity::Medium.to_string(), "Medium");
        assert_eq!(Severity::Medium.emoji(), "🟡");

        assert_eq!(Severity::Low.to_string(), "Low");
        assert_eq!(Severity::Low.emoji(), "🟢");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert_eq!(Severity::Critical.rank(), 4);
        assert_eq!(Severity::Low.rank(), 1);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!("critical".parse::<Severity>().unwrap(), Severity::Critical);
        assert_eq!("crit".parse::<Severity>().unwrap(), Severity::Critical);
        assert_eq!("high".parse::<Severity>().unwrap(), Severity::High);
        assert_eq!("medium".parse::<Severity>().unwrap(), Severity::Medium);
        assert_eq!("med".parse::<Severity>().unwrap(), Severity::Medium);
        assert_eq!("low".parse::<Severity>().unwrap(), Severity::Low);
        assert!("invalid".parse::<Severity>().is_err());
    }

    #[test]
    fn test_filter_debt_by_min_severity() {
        let now = Utc::now();
        let items = vec![
            DebtItem {
                kind: DebtKind::AbandonedSession,
                title: "Low item".to_string(),
                description: "".to_string(),
                severity: Severity::Low,
                timestamp: now,
                entity_id: None,
            },
            DebtItem {
                kind: DebtKind::OpenRefactor,
                title: "High item".to_string(),
                description: "".to_string(),
                severity: Severity::High,
                timestamp: now,
                entity_id: None,
            },
            DebtItem {
                kind: DebtKind::StaleDecision,
                title: "Critical item".to_string(),
                description: "".to_string(),
                severity: Severity::Critical,
                timestamp: now,
                entity_id: None,
            },
        ];

        let high_and_above = filter_debt_by_min_severity(&items, Severity::High);
        assert_eq!(high_and_above.len(), 2);
        assert!(high_and_above.iter().all(|d| d.severity >= Severity::High));

        let critical_only = filter_debt_by_min_severity(&items, Severity::Critical);
        assert_eq!(critical_only.len(), 1);
        assert_eq!(critical_only[0].severity, Severity::Critical);
    }

    #[test]
    fn test_debt_kind_from_str() {
        assert_eq!("abandoned_session".parse::<DebtKind>().unwrap(), DebtKind::AbandonedSession);
        assert_eq!("abandoned".parse::<DebtKind>().unwrap(), DebtKind::AbandonedSession);
        assert_eq!("unresolved_query".parse::<DebtKind>().unwrap(), DebtKind::UnresolvedQuery);
        assert_eq!("unresolved".parse::<DebtKind>().unwrap(), DebtKind::UnresolvedQuery);
        assert_eq!("stale_decision".parse::<DebtKind>().unwrap(), DebtKind::StaleDecision);
        assert_eq!("open_refactor".parse::<DebtKind>().unwrap(), DebtKind::OpenRefactor);
        assert_eq!("todo_without_action".parse::<DebtKind>().unwrap(), DebtKind::TodoWithoutAction);
        assert_eq!("todo".parse::<DebtKind>().unwrap(), DebtKind::TodoWithoutAction);
        assert!("unknown_kind".parse::<DebtKind>().is_err());
    }

    #[test]
    fn test_filter_debt_by_kind() {
        let now = Utc::now();
        let items = vec![
            DebtItem {
                kind: DebtKind::AbandonedSession,
                title: "Abandoned".to_string(),
                description: "".to_string(),
                severity: Severity::Low,
                timestamp: now,
                entity_id: None,
            },
            DebtItem {
                kind: DebtKind::StaleDecision,
                title: "Stale".to_string(),
                description: "".to_string(),
                severity: Severity::High,
                timestamp: now,
                entity_id: None,
            },
        ];

        let filtered = filter_debt_by_kind(&items, &DebtKind::AbandonedSession);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Abandoned");

        let none_found = filter_debt_by_kind(&items, &DebtKind::OpenRefactor);
        assert!(none_found.is_empty());
    }

    #[test]
    fn test_debt_item_serde() {
        let now = Utc::now();
        let item = DebtItem {
            kind: DebtKind::OpenRefactor,
            title: "WIP".to_string(),
            description: "Desc".to_string(),
            severity: Severity::Critical,
            timestamp: now,
            entity_id: Some("ent-123".to_string()),
        };

        let serialized = serde_json::to_string(&item).unwrap();
        let deserialized: DebtItem = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, item);
    }
}

