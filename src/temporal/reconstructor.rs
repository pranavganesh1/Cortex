use super::TimeRange;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WorkSession {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_minutes: i64,
    pub files_touched: Vec<String>,
    pub commits: Vec<String>,
    pub queries: Vec<String>,
    pub decisions: Vec<String>,
    pub active_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MentalState {
    pub timestamp: DateTime<Utc>,
    pub session: WorkSession,
    pub active_file: Option<String>,
    pub recent_commits: Vec<String>,
    pub recent_decisions: Vec<String>,
    pub recent_queries: Vec<String>,
    pub files_in_session: Vec<String>,
}

#[derive(sqlx::FromRow)]
struct EventRow {
    entity_id: String,
    action: String,
    timestamp: String,
    metadata: Option<String>,
}

pub async fn get_timeline(pool: &SqlitePool, range: &TimeRange) -> Result<Vec<WorkSession>> {
    let events: Vec<EventRow> = sqlx::query_as(
        r#"SELECT entity_id, action, timestamp, metadata FROM events
           WHERE timestamp >= ? AND timestamp <= ?
           ORDER BY timestamp ASC"#
    )
    .bind(range.start.to_rfc3339())
    .bind(range.end.to_rfc3339())
    .fetch_all(pool)
    .await?;

    let parsed_events: Vec<(DateTime<Utc>, String, String, Option<String>)> = events.into_iter()
        .filter_map(|e| {
            let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?.with_timezone(&Utc);
            Some((ts, e.entity_id, e.action, e.metadata))
        })
        .collect();

    if parsed_events.is_empty() {
        return Ok(vec![]);
    }

    // Group into sessions by 30-minute gaps
    let mut sessions: Vec<WorkSession> = Vec::new();
    let mut current_session: Option<(DateTime<Utc>, Vec<(DateTime<Utc>, String, String, Option<String>)>)> = None;

    for evt in parsed_events {
        match &mut current_session {
            None => current_session = Some((evt.0, vec![evt])),
            Some((_start, buffer)) => {
                if evt.0 - buffer.last().unwrap().0 > Duration::minutes(30) {
                    // Finalize previous session
                    sessions.push(build_session(buffer));
                    current_session = Some((evt.0, vec![evt]));
                } else {
                    buffer.push(evt);
                }
            }
        }
    }

    if let Some((_, buffer)) = current_session {
        sessions.push(build_session(&buffer));
    }

    Ok(sessions)
}

fn build_session(events: &[(DateTime<Utc>, String, String, Option<String>)]) -> WorkSession {
    let start = events.first().unwrap().0;
    let end = events.last().unwrap().0;
    let duration = (end - start).num_minutes();

    let mut files = std::collections::HashSet::new();
    let mut commits = Vec::new();
    let mut queries = Vec::new();
    let mut decisions = Vec::new();
    let mut file_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (_, entity_id, action, metadata) in events {
        match action.as_str() {
            "file:save" => {
                files.insert(entity_id.clone());
                *file_counts.entry(entity_id.clone()).or_insert(0) += 1;
            }
            "git:commit" => commits.push(entity_id.clone()),
            "query:asked" => {
                if let Some(meta) = metadata {
                    queries.push(meta.clone());
                }
            }
            "decision:extracted" => {
                if let Some(meta) = metadata {
                    decisions.push(meta.clone());
                }
            }
            _ => {}
        }
    }

    let active_file = file_counts.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(name, _)| name.clone())
        .or_else(|| files.iter().next().cloned());

    WorkSession {
        start,
        end,
        duration_minutes: duration,
        files_touched: files.into_iter().collect(),
        commits,
        queries,
        decisions,
        active_file,
    }
}

pub async fn reconstruct_state(pool: &SqlitePool, point: DateTime<Utc>) -> Result<MentalState> {
    // Find the session containing this point
    let window_start = point - Duration::hours(4);
    let window_end = point + Duration::minutes(30);

    let events: Vec<EventRow> = sqlx::query_as(
        r#"SELECT entity_id, action, timestamp, metadata FROM events
           WHERE timestamp >= ? AND timestamp <= ?
           ORDER BY timestamp ASC"#
    )
    .bind(window_start.to_rfc3339())
    .bind(window_end.to_rfc3339())
    .fetch_all(pool)
    .await?;

    let parsed: Vec<(DateTime<Utc>, String, String, Option<String>)> = events.into_iter()
        .filter_map(|e| {
            let ts = DateTime::parse_from_rfc3339(&e.timestamp).ok()?.with_timezone(&Utc);
            Some((ts, e.entity_id, e.action, e.metadata))
        })
        .collect();

    // Find which session contains the point
    let mut session_events = Vec::new();
    let mut in_session = false;
    let mut last_time = point - Duration::hours(4);

    for evt in &parsed {
        if evt.0 > point && !in_session {
            break;
        }
        if evt.0 - last_time > Duration::minutes(30) && in_session {
            // New session started before point, reset
            session_events.clear();
            in_session = false;
        }
        if !in_session && (point - evt.0).num_minutes().abs() <= 120 {
            in_session = true;
        }
        if in_session {
            session_events.push(evt.clone());
        }
        last_time = evt.0;
    }

    let session = build_session(&session_events);

    // Find active file at exact point (most recent file save before point)
    let active_file = parsed.iter()
        .filter(|(ts, _, action, _)| *action == "file:save" && *ts <= point)
        .last()
        .map(|(_, entity_id, _, _)| entity_id.clone())
        .or_else(|| session.active_file.clone());

    // Recent commits within 2 hours of point
    let recent_commits: Vec<String> = parsed.iter()
        .filter(|(ts, _, action, _)| *action == "git:commit" && (point - *ts).num_minutes().abs() <= 120)
        .map(|(_, entity_id, _, _)| entity_id.clone())
        .collect();

    // Recent decisions
    let recent_decisions: Vec<String> = parsed.iter()
        .filter(|(ts, _, action, _)| *action == "decision:extracted" && (point - *ts).num_minutes().abs() <= 120)
        .filter_map(|(_, _, _, meta)| meta.clone())
        .collect();

    // Recent queries
    let recent_queries: Vec<String> = parsed.iter()
        .filter(|(ts, _, action, _)| *action == "query:asked" && (point - *ts).num_minutes().abs() <= 120)
        .filter_map(|(_, _, _, meta)| meta.clone())
        .collect();

    Ok(MentalState {
        timestamp: point,
        session,
        active_file,
        recent_commits,
        recent_decisions,
        recent_queries,
        files_in_session: session_events.iter()
            .filter(|(_, _, action, _)| action == "file:save")
            .map(|(_, entity_id, _, _)| entity_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect(),
    })
}
