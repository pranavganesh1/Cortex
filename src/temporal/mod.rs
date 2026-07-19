pub mod parser;
pub mod reconstructor;
pub mod formatter;

#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub is_point: bool,
}
