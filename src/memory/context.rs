use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFrame {
    pub id: String,
    pub topic: String,              // auto-generated or inferred
    pub files: Vec<String>,         // files in this context
    pub decisions: Vec<String>,     // decision texts
    pub queries: Vec<String>,       // questions asked
    pub commits: Vec<String>,       // commit hashes
    pub start_time: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_active: bool,
}

impl ContextFrame {
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            topic: "untitled".to_string(),
            files: Vec::new(),
            decisions: Vec::new(),
            queries: Vec::new(),
            commits: Vec::new(),
            start_time: now,
            last_active: now,
            is_active: true,
        }
    }

    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }

    pub fn add_file(&mut self, path: &str) {
        if !self.files.contains(&path.to_string()) {
            self.files.push(path.to_string());
            if self.files.len() <= 3 {
                self.infer_topic();
            }
        }
        self.touch();
    }

    pub fn add_decision(&mut self, text: &str) {
        self.decisions.push(text.to_string());
        self.touch();
    }

    pub fn add_query(&mut self, text: &str) {
        self.queries.push(text.to_string());
        self.touch();
    }

    pub fn add_commit(&mut self, hash: &str) {
        self.commits.push(hash.to_string());
        self.touch();
    }

    fn infer_topic(&mut self) {
        // Infer topic from common directory or file names
        if self.files.is_empty() {
            return;
        }

        let normalized: Vec<String> = self.files.iter().map(|f| f.replace('\\', "/")).collect();
        // Find longest common path prefix
        let parts: Vec<Vec<&str>> = normalized.iter()
            .map(|f| f.split('/').collect())
            .collect();

        if parts.is_empty() {
            return;
        }

        let mut common_len = 0;
        'outer: for i in 0..parts[0].len() {
            let val = parts[0][i];
            for p in &parts[1..] {
                if p.get(i) != Some(&val) {
                    break 'outer;
                }
            }
            common_len = i + 1;
        }

        if common_len > 0 {
            let topic = parts[0][..common_len.min(parts[0].len())].join("/");
            self.topic = topic;
        } else {
            // Use filename stem
            let stem = std::path::Path::new(&self.files[0])
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "untitled".to_string());
            self.topic = stem;
        }
    }

    pub fn duration_minutes(&self) -> i64 {
        (self.last_active - self.start_time).num_minutes()
    }

    pub fn is_stale(&self, now: DateTime<Utc>, threshold_minutes: i64) -> bool {
        (now - self.last_active).num_minutes() > threshold_minutes
    }

    pub fn is_related_to_file(&self, path: &str) -> bool {
        // Related if same directory or same file
        let norm_path = path.replace('\\', "/");
        let path_parts: Vec<&str> = norm_path.split('/').collect();
        for file in &self.files {
            let norm_file = file.replace('\\', "/");
            let file_parts: Vec<&str> = norm_file.split('/').collect();
            // Check if they share a parent directory
            let min_len = path_parts.len().min(file_parts.len()).saturating_sub(1);
            if min_len > 0 && path_parts[..min_len] == file_parts[..min_len] {
                return true;
            }
            // Or if it's literally the same file
            if norm_file == norm_path {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_context_frame_creation_and_touch() {
        let mut frame = ContextFrame::new("test-1".to_string());
        assert_eq!(frame.id, "test-1");
        assert_eq!(frame.topic, "untitled");
        assert!(frame.files.is_empty());

        let prev = frame.last_active;
        frame.touch();
        assert!(frame.last_active >= prev);
    }

    #[test]
    fn test_add_items() {
        let mut frame = ContextFrame::new("test-2".to_string());
        frame.add_file("src/lib.rs");
        frame.add_decision("use sqlite");
        frame.add_query("how to query?");
        frame.add_commit("abcdef1");

        assert_eq!(frame.files, vec!["src/lib.rs"]);
        assert_eq!(frame.decisions, vec!["use sqlite"]);
        assert_eq!(frame.queries, vec!["how to query?"]);
        assert_eq!(frame.commits, vec!["abcdef1"]);
    }

    #[test]
    fn test_infer_topic_common_prefix() {
        let mut frame = ContextFrame::new("test-3".to_string());
        frame.add_file("src/api/routes.rs");
        frame.add_file("src/api/context.rs");

        assert_eq!(frame.topic, "src/api");
    }

    #[test]
    fn test_infer_topic_windows_paths() {
        let mut frame = ContextFrame::new("test-4".to_string());
        frame.add_file("src\\db\\models.rs");
        frame.add_file("src\\db\\schema.rs");

        assert_eq!(frame.topic, "src/db");
    }

    #[test]
    fn test_is_related_to_file() {
        let mut frame = ContextFrame::new("test-5".to_string());
        frame.add_file("src/core/models.rs");

        assert!(frame.is_related_to_file("src/core/events.rs"));
        assert!(frame.is_related_to_file("src\\core\\events.rs"));
        assert!(frame.is_related_to_file("src/core/models.rs"));
        assert!(!frame.is_related_to_file("tests/integration.rs"));
    }

    #[test]
    fn test_is_stale() {
        let mut frame = ContextFrame::new("test-6".to_string());
        let now = Utc::now();
        frame.last_active = now - Duration::minutes(20);

        assert!(frame.is_stale(now, 15));
        assert!(!frame.is_stale(now, 30));
    }
}
