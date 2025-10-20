use serde::{Deserialize, Serialize};
use std::fmt;

/// Recording ID that uniquely identifies a recording session
/// Format: {stream}/{timestamp}
/// where timestamp is a 10-digit Unix timestamp (seconds since epoch)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordingId {
    /// Stream name
    pub stream: String,
    /// Unix timestamp (seconds) when the recording started
    pub timestamp: i64,
}

impl RecordingId {
    /// Create a new RecordingId with the current timestamp
    pub fn new(stream: &str) -> Self {
        Self {
            stream: stream.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a RecordingId with a specific timestamp
    pub fn with_timestamp(stream: &str, timestamp: i64) -> Self {
        Self {
            stream: stream.to_string(),
            timestamp,
        }
    }

    /// Generate the path prefix for storage
    /// Format: {stream}/{timestamp}
    /// Example: "camera01/1729411200"
    pub fn path_prefix(&self) -> String {
        format!("{}/{}", self.stream, self.timestamp)
    }

    /// Parse a RecordingId from a path prefix
    /// Returns None if the path format is invalid
    pub fn from_path(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            let timestamp = parts[1].parse::<i64>().ok()?;
            Some(Self {
                stream: parts[0].to_string(),
                timestamp,
            })
        } else {
            None
        }
    }

    /// Get a human-readable representation of the recording start time
    pub fn start_time(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.timestamp, 0).unwrap_or_else(chrono::Utc::now)
    }

    /// Validate that the timestamp is reasonable (not too far in past or future)
    pub fn is_valid(&self) -> bool {
        if self.stream.is_empty() {
            return false;
        }

        let now = chrono::Utc::now().timestamp();
        // Allow timestamps from 2020-01-01 to 100 years in the future
        let min_timestamp = 1577836800; // 2020-01-01
        let max_timestamp = now + (100 * 365 * 24 * 60 * 60); // 100 years from now

        self.timestamp >= min_timestamp && self.timestamp <= max_timestamp
    }

    /// Check if this recording is older than the specified duration
    pub fn is_older_than(&self, duration: chrono::Duration) -> bool {
        let now = chrono::Utc::now();
        let recording_time = self.start_time();
        now.signed_duration_since(recording_time) > duration
    }
}

impl fmt::Display for RecordingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.stream, self.timestamp)
    }
}

impl From<RecordingId> for String {
    fn from(id: RecordingId) -> Self {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_recording_id() {
        let id = RecordingId::new("camera01");
        assert_eq!(id.stream, "camera01");
        assert!(id.timestamp > 0);
        assert!(id.is_valid());
    }

    #[test]
    fn test_with_timestamp() {
        let timestamp = 1729411200; // 2024-10-20 08:00:00 UTC
        let id = RecordingId::with_timestamp("camera01", timestamp);
        assert_eq!(id.stream, "camera01");
        assert_eq!(id.timestamp, timestamp);
    }

    #[test]
    fn test_path_prefix() {
        let id = RecordingId::with_timestamp("camera01", 1729411200);
        assert_eq!(id.path_prefix(), "camera01/1729411200");
    }

    #[test]
    fn test_from_path_valid() {
        let path = "camera01/1729411200";
        let id = RecordingId::from_path(path).unwrap();
        assert_eq!(id.stream, "camera01");
        assert_eq!(id.timestamp, 1729411200);
    }

    #[test]
    fn test_from_path_with_extra_segments() {
        let path = "camera01/1729411200/manifest.mpd";
        let id = RecordingId::from_path(path).unwrap();
        assert_eq!(id.stream, "camera01");
        assert_eq!(id.timestamp, 1729411200);
    }

    #[test]
    fn test_from_path_invalid() {
        assert!(RecordingId::from_path("").is_none());
        assert!(RecordingId::from_path("camera01").is_none());
        assert!(RecordingId::from_path("camera01/invalid").is_none());
    }

    #[test]
    fn test_start_time() {
        let timestamp = 1729411200; // 2024-10-20 08:00:00 UTC
        let id = RecordingId::with_timestamp("camera01", timestamp);
        let start_time = id.start_time();
        assert_eq!(start_time.timestamp(), timestamp);
    }

    #[test]
    fn test_is_valid() {
        // Valid timestamp
        let valid_id = RecordingId::with_timestamp("camera01", 1729411200);
        assert!(valid_id.is_valid());

        // Too old (before 2020)
        let old_id = RecordingId::with_timestamp("camera01", 1000000000);
        assert!(!old_id.is_valid());

        // Empty stream
        let empty_stream = RecordingId::with_timestamp("", 1729411200);
        assert!(!empty_stream.is_valid());
    }

    #[test]
    fn test_is_older_than() {
        let now = chrono::Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let id = RecordingId::with_timestamp("camera01", one_hour_ago.timestamp());

        assert!(id.is_older_than(chrono::Duration::minutes(30)));
        assert!(!id.is_older_than(chrono::Duration::hours(2)));
    }

    #[test]
    fn test_display() {
        let id = RecordingId::with_timestamp("camera01", 1729411200);
        assert_eq!(id.to_string(), "camera01/1729411200");
    }

    #[test]
    fn test_serde() {
        let id = RecordingId::with_timestamp("camera01", 1729411200);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: RecordingId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
