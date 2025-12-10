use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMeta {
    pub start_time: i64, // Unix timestamp (seconds)
    pub end_time: i64,
    pub duration: f64, // seconds
    pub size: u64,     // bytes
    pub video_codec: String,
    pub audio_codec: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingIndexItem {
    pub stream_id: String,
    pub start_time: i64,
    pub duration: f64,
    pub path: String, // Relative path: "{timestamp_start}/"
    #[serde(default)]
    pub status: RecordingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecordingStatus {
    Recording,
    LocalSaved,
    Syncing,
    Synced,
    Purged,
}

impl Default for RecordingStatus {
    fn default() -> Self {
        RecordingStatus::LocalSaved
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordingIndex {
    pub items: Vec<RecordingIndexItem>,
}
