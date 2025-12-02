use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{error, info};

use crate::config::Config;

pub mod config;

#[derive(Clone)]
pub struct AppState {
    config: Config,
}

#[derive(Debug, Serialize)]
struct StreamInfo {
    id: String,
    record_count: usize,
    latest_record: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexJson {
    ver: u32,
    id: String,
    stream: String,
    start_time: i64,
    end_time: i64,
    duration: u64,
    status: String,
    #[serde(default)]
    files: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RecordingInfo {
    id: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_sec: f64,
    status: String,
    playback_url: String,
}

pub async fn serve(
    config: Config,
    listener: tokio::net::TcpListener,
    signal: impl std::future::Future<Output = ()> + Send + 'static,
) {
    let state = AppState {
        config: config.clone(),
    };

    let record_path = PathBuf::from(&config.storage.path);
    if !record_path.exists() {
        info!("creating recording directory: {:?}", record_path);
        if let Err(e) = fs::create_dir_all(&record_path) {
            error!("failed to create recording directory: {}", e);
        }
    }

    let cors = if config.http.cors {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
    };

    let app = Router::new()
        .route("/api/vod/streams", get(list_streams))
        .route("/api/vod/streams/:stream_id", get(list_recordings))
        .nest_service("/vod", ServeDir::new(&config.storage.path))
        .layer(cors)
        .with_state(state);

    info!("livevod listening on {}", config.http.listen);

    axum::serve(listener, app)
        .with_graceful_shutdown(signal)
        .await
        .unwrap();
}

async fn list_streams(State(state): State<AppState>) -> Json<Vec<StreamInfo>> {
    let root = PathBuf::from(&state.config.storage.path);
    let mut streams = Vec::new();

    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let stream_id = entry.file_name().to_string_lossy().to_string();
                    
                    // Simple stats: count subdirs which look like timestamps
                    let (count, latest) = get_stream_stats(&entry.path());
                    
                    streams.push(StreamInfo {
                        id: stream_id,
                        record_count: count,
                        latest_record: latest,
                    });
                }
            }
        }
    }
    
    // Sort by latest activity
    streams.sort_by(|a, b| b.latest_record.cmp(&a.latest_record));

    Json(streams)
}

fn get_stream_stats(path: &PathBuf) -> (usize, Option<DateTime<Utc>>) {
    let mut count = 0;
    let mut latest_ts: Option<i64> = None;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Check if name is a 10-digit timestamp
                    if name.len() == 10 && name.chars().all(|c| c.is_ascii_digit()) {
                        count += 1;
                        if let Ok(ts) = name.parse::<i64>() {
                            latest_ts = Some(latest_ts.map_or(ts, |v| v.max(ts)));
                        }
                    }
                }
            }
        }
    }

    let latest_date = latest_ts.and_then(|ts| DateTime::from_timestamp(ts, 0));
    (count, latest_date)
}

async fn list_recordings(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
) -> Result<Json<Vec<RecordingInfo>>, StatusCode> {
    let mut path = PathBuf::from(&state.config.storage.path);
    path.push(&stream_id);

    if !path.exists() || !path.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut recordings = Vec::new();

    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Check if folder is a timestamp
                    if name.len() == 10 && name.chars().all(|c| c.is_ascii_digit()) {
                        let index_path = entry.path().join("index.json");
                        if index_path.exists() {
                            if let Ok(content) = fs::read_to_string(&index_path) {
                                if let Ok(meta) = serde_json::from_str::<IndexJson>(&content) {
                                    let start = DateTime::from_timestamp_millis(meta.start_time)
                                        .unwrap_or_default();
                                    let end = DateTime::from_timestamp_millis(meta.end_time)
                                        .unwrap_or_default();
                                    
                                    recordings.push(RecordingInfo {
                                        id: meta.id,
                                        start_time: start,
                                        end_time: end,
                                        duration_sec: meta.duration as f64 / 1000.0,
                                        status: meta.status,
                                        playback_url: format!("/vod/{}/{}/manifest.mpd", stream_id, name),
                                    });
                                }
                            }
                        } else {
                            // Fallback if index.json is missing but folder exists (legacy or broken)
                             if let Ok(ts) = name.parse::<i64>() {
                                 let start = DateTime::from_timestamp(ts, 0).unwrap_or_default();
                                 recordings.push(RecordingInfo {
                                     id: name.clone(),
                                     start_time: start,
                                     end_time: start, // Unknown
                                     duration_sec: 0.0,
                                     status: "unknown".to_string(),
                                     playback_url: format!("/vod/{}/{}/manifest.mpd", stream_id, name),
                                 });
                             }
                        }
                    }
                }
            }
        }
    }

    // Sort by start time descending
    recordings.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    Ok(Json(recordings))
}
