use super::TimeRange;
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc, Weekday};
use regex::Regex;

pub fn parse_time_expression(expr: &str) -> Option<TimeRange> {
    let lower = expr.trim().to_lowercase();
    let now = Utc::now();

    // Exact matches
    match lower.as_str() {
        "today" | "now" => return Some(TimeRange {
            start: now.date_naive().and_hms_opt(0, 0, 0)?.and_local_timezone(Utc).single()?,
            end: now,
            is_point: false,
        }),
        "yesterday" => {
            let yest = now - Duration::days(1);
            return Some(TimeRange {
                start: yest.date_naive().and_hms_opt(0, 0, 0)?.and_local_timezone(Utc).single()?,
                end: yest.date_naive().and_hms_opt(23, 59, 59)?.and_local_timezone(Utc).single()?,
                is_point: false,
            });
        }
        "this morning" => return Some(TimeRange {
            start: now.date_naive().and_hms_opt(6, 0, 0)?.and_local_timezone(Utc).single()?,
            end: now.date_naive().and_hms_opt(12, 0, 0)?.and_local_timezone(Utc).single()?,
            is_point: false,
        }),
        "this afternoon" => return Some(TimeRange {
            start: now.date_naive().and_hms_opt(12, 0, 0)?.and_local_timezone(Utc).single()?,
            end: now.date_naive().and_hms_opt(18, 0, 0)?.and_local_timezone(Utc).single()?,
            is_point: false,
        }),
        "this evening" => return Some(TimeRange {
            start: now.date_naive().and_hms_opt(18, 0, 0)?.and_local_timezone(Utc).single()?,
            end: now.date_naive().and_hms_opt(23, 59, 59)?.and_local_timezone(Utc).single()?,
            is_point: false,
        }),
        "last week" => return Some(TimeRange {
            start: (now - Duration::days(7)).date_naive().and_hms_opt(0, 0, 0)?.and_local_timezone(Utc).single()?,
            end: now,
            is_point: false,
        }),
        "last month" => return Some(TimeRange {
            start: (now - Duration::days(30)).date_naive().and_hms_opt(0, 0, 0)?.and_local_timezone(Utc).single()?,
            end: now,
            is_point: false,
        }),
        _ => {}
    }

    // "last N days/hours/minutes"
    let re = Regex::new(r"last\s+(\d+)\s+(day|days|hour|hours|minute|minutes|min|mins)").ok()?;
    if let Some(cap) = re.captures(&lower) {
        let n: i64 = cap.get(1)?.as_str().parse().ok()?;
        let unit = cap.get(2)?.as_str();
        let duration = match unit {
            "day" | "days" => Duration::days(n),
            "hour" | "hours" => Duration::hours(n),
            _ => Duration::minutes(n),
        };
        return Some(TimeRange {
            start: now - duration,
            end: now,
            is_point: false,
        });
    }

    // "N days/hours/minutes ago" (point in time)
    let re2 = Regex::new(r"(\d+)\s+(day|days|hour|hours|minute|minutes|min|mins)\s+ago").ok()?;
    if let Some(cap) = re2.captures(&lower) {
        let n: i64 = cap.get(1)?.as_str().parse().ok()?;
        let unit = cap.get(2)?.as_str();
        let duration = match unit {
            "day" | "days" => Duration::days(n),
            "hour" | "hours" => Duration::hours(n),
            _ => Duration::minutes(n),
        };
        let point = now - duration;
        return Some(TimeRange {
            start: point - Duration::minutes(30),
            end: point + Duration::minutes(30),
            is_point: true,
        });
    }

    // Day of week: "monday", "tuesday", etc.
    if let Some(weekday) = parse_weekday(&lower) {
        let date = most_recent_weekday(weekday, now);
        
        // Check for time: "tuesday 3pm", "monday 14:30"
        if let Some(time) = parse_time_of_day(&lower) {
            let dt = date.and_time(time).and_local_timezone(Utc).single()?;
            // If in future, go back one week
            let dt = if dt > now { dt - Duration::days(7) } else { dt };
            return Some(TimeRange {
                start: dt - Duration::minutes(30),
                end: dt + Duration::minutes(30),
                is_point: true,
            });
        }
        
        return Some(TimeRange {
            start: date.and_hms_opt(0, 0, 0)?.and_local_timezone(Utc).single()?,
            end: date.and_hms_opt(23, 59, 59)?.and_local_timezone(Utc).single()?,
            is_point: false,
        });
    }

    // Just time: "3pm", "15:00"
    if let Some(time) = parse_time_of_day(&lower) {
        let mut dt = now.date_naive().and_time(time).and_local_timezone(Utc).single()?;
        if dt > now {
            dt = dt - Duration::days(1);
        }
        return Some(TimeRange {
            start: dt - Duration::minutes(30),
            end: dt + Duration::minutes(30),
            is_point: true,
        });
    }

    None
}

fn parse_weekday(text: &str) -> Option<Weekday> {
    let days = [
        ("monday", Weekday::Mon), ("mon", Weekday::Mon),
        ("tuesday", Weekday::Tue), ("tue", Weekday::Tue), ("tues", Weekday::Tue),
        ("wednesday", Weekday::Wed), ("wed", Weekday::Wed),
        ("thursday", Weekday::Thu), ("thu", Weekday::Thu), ("thurs", Weekday::Thu),
        ("friday", Weekday::Fri), ("fri", Weekday::Fri),
        ("saturday", Weekday::Sat), ("sat", Weekday::Sat),
        ("sunday", Weekday::Sun), ("sun", Weekday::Sun),
    ];
    for (name, wd) in days {
        if text.contains(name) {
            return Some(wd);
        }
    }
    None
}

fn most_recent_weekday(target: Weekday, now: DateTime<Utc>) -> NaiveDate {
    let today = now.date_naive();
    for days_back in 0..=7 {
        let candidate = today - Duration::days(days_back);
        if candidate.weekday() == target {
            return candidate;
        }
    }
    today
}

fn parse_time_of_day(text: &str) -> Option<NaiveTime> {
    // "3pm", "3:30pm", "15:00", "3:30 PM"
    let re = Regex::new(r"(?i)(\d{1,2})(?::(\d{2}))?\s*(am|pm)?").ok()?;
    let cap = re.captures(text)?;
    
    let hour: u32 = cap.get(1)?.as_str().parse().ok()?;
    let minute: u32 = cap.get(2).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
    let ampm = cap.get(3).map(|m| m.as_str().to_lowercase());
    
    let hour = match ampm.as_deref() {
        Some("pm") if hour < 12 => hour + 12,
        Some("am") if hour == 12 => 0,
        _ => hour,
    };
    
    NaiveTime::from_hms_opt(hour, minute, 0)
}
