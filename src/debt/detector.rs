use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum DebtKind {
    AbandonedSession,
    UnresolvedQuery,
    StaleDecision,
    OpenRefactor,
    TodoWithoutAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DebtItem {
    pub kind: DebtKind,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub timestamp: DateTime<Utc>,
    pub entity_id: Option<String>,
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
        let sev = |s: &Severity| match s {
            Severity::Critical => 0,
            Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
        };
        let sev_cmp = sev(&a.severity).cmp(&sev(&b.severity));
        if sev_cmp != std::cmp::Ordering::Equal {
            return sev_cmp;
        }
        b.timestamp.cmp(&a.timestamp)
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
