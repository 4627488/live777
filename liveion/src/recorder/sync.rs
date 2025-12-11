use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, Result, anyhow};
use futures::TryStreamExt;
use opendal::{EntryMode, ErrorKind, Operator};
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use tokio::time::sleep;

use crate::{
    config::{RecorderConfig, RecorderSyncConfig},
    recorder::meta::{RecordingIndex, RecordingIndexItem, RecordingMeta, RecordingStatus},
};

const INDEX_FILENAME: &str = "index.json";
const META_FILENAME: &str = "meta.json";
const MANIFEST_FILENAME: &str = "manifest.mpd";

pub fn spawn(operator: Operator, cfg: Arc<RecorderConfig>) {
    let sync_cfg = cfg.sync.clone();
    if !sync_cfg.enabled {
        tracing::info!("[recorder] cloud sync disabled");
        return;
    }

    if sync_cfg.liveman_url.trim().is_empty() {
        tracing::warn!("[recorder] cloud sync enabled but liveman_url is empty");
        return;
    }

    match SyncWorker::new(operator, sync_cfg) {
        Ok(worker) => {
            tracing::info!("[recorder] cloud sync worker started");
            tokio::spawn(async move {
                worker.run().await;
            });
        }
        Err(err) => {
            tracing::error!("[recorder] failed to initialize sync worker: {err}");
        }
    }
}

struct SyncWorker {
    operator: Operator,
    client: Client,
    cfg: RecorderSyncConfig,
    base_url: String,
}

impl SyncWorker {
    fn new(operator: Operator, cfg: RecorderSyncConfig) -> Result<Self> {
        let client = Client::builder().build()?;
        let base_url = cfg.liveman_url.trim_end_matches('/').to_string();
        Ok(Self {
            operator,
            client,
            base_url,
            cfg,
        })
    }

    async fn run(self) {
        let interval = Duration::from_secs(self.cfg.interval_seconds.max(5));
        loop {
            if let Err(err) = self.sync_once().await {
                tracing::warn!("[recorder] cloud sync iteration failed: {err:?}");
            }
            sleep(interval).await;
        }
    }

    async fn sync_once(&self) -> Result<()> {
        let streams = self.list_streams().await?;
        for stream in streams {
            if let Err(err) = self.sync_stream(&stream).await {
                tracing::warn!("[recorder] sync stream {stream} failed: {err:?}");
            }
        }
        Ok(())
    }

    async fn list_streams(&self) -> Result<Vec<String>> {
        let mut result = Vec::new();
        let mut lister = self.operator.lister("").await?;
        while let Some(entry) = lister.try_next().await? {
            if matches!(entry.metadata().mode(), EntryMode::DIR) {
                let path = entry.path().trim_end_matches('/');
                if !path.is_empty() {
                    result.push(path.to_string());
                }
            }
        }
        Ok(result)
    }

    async fn sync_stream(&self, stream: &str) -> Result<()> {
        let Some(mut index) = self.load_index(stream).await? else {
            return Ok(());
        };

        if index.items.is_empty() {
            return Ok(());
        }

        let mut processed = 0usize;
        let mut i = 0;
        while i < index.items.len() {
            if !matches!(
                index.items[i].status,
                RecordingStatus::LocalSaved | RecordingStatus::Syncing
            ) {
                i += 1;
                continue;
            }

            if self.cfg.batch_limit > 0 && processed >= self.cfg.batch_limit {
                break;
            }

            // Clone the item for processing to avoid borrow checker issues
            let item_clone = index.items[i].clone();

            index.items[i].status = RecordingStatus::Syncing;
            self.save_index(stream, &index).await?;

            match self.sync_entry(stream, &item_clone).await {
                Ok(_) => {
                    index.items[i].status = RecordingStatus::Synced;
                    processed += 1;
                }
                Err(err) => {
                    tracing::warn!(
                        "[recorder] failed to sync {stream}/{}: {err:?}",
                        item_clone.path
                    );
                    index.items[i].status = RecordingStatus::LocalSaved;
                }
            }

            self.save_index(stream, &index).await?;
            i += 1;
        }

        Ok(())
    }

    async fn sync_entry(&self, stream: &str, item: &RecordingIndexItem) -> Result<()> {
        let record_dir = normalize_record_dir(&item.path)?;
        let files = self.collect_record_files(stream, &record_dir).await?;
        if files.is_empty() {
            return Err(anyhow!("no files found for {stream}/{record_dir}"));
        }

        for relative in files {
            self.upload_file(stream, &relative).await?;
        }

        self.confirm_record(stream, item, &record_dir).await?;
        Ok(())
    }

    async fn collect_record_files(&self, stream: &str, record_dir: &str) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let base = format!("{stream}/{record_dir}/");
        let stream_prefix = format!("{stream}/");
        let mut lister = self.operator.lister_with(&base).recursive(true).await?;
        while let Some(entry) = lister.try_next().await? {
            match entry.metadata().mode() {
                EntryMode::FILE => {
                    if let Some(relative) = entry.path().strip_prefix(&stream_prefix) {
                        files.push(relative.to_string());
                    }
                }
                EntryMode::DIR | EntryMode::Unknown => {}
            }
        }
        Ok(files)
    }

    async fn upload_file(&self, stream: &str, relative_path: &str) -> Result<()> {
        let clean_relative = relative_path.trim_start_matches('/');
        let local_path = format!("{stream}/{clean_relative}");
        let body = self
            .operator
            .read(&local_path)
            .await
            .with_context(|| format!("read {} from storage", local_path))?;

        let presign_req = PresignUploadPayload {
            stream_id: stream.to_string(),
            filename: clean_relative.to_string(),
            method: "PUT".to_string(),
        };

        let presign_resp: PresignUploadResponse = self
            .authed(
                self.client
                    .post(self.endpoint(api::path::storage_presign_upload())),
            )
            .json(&presign_req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let method = Method::from_bytes(presign_resp.method.as_bytes())?;
        let mut upload = self.client.request(method, presign_resp.url);
        for (key, value) in presign_resp.headers {
            upload = upload.header(key, value);
        }

        upload
            .body(body.to_bytes())
            .send()
            .await?
            .error_for_status()
            .with_context(|| format!("upload {clean_relative} to cloud"))?;

        Ok(())
    }

    async fn confirm_record(
        &self,
        stream: &str,
        item: &RecordingIndexItem,
        record_dir: &str,
    ) -> Result<()> {
        let meta = self.read_meta(stream, record_dir).await?;
        let meta_value = meta
            .as_ref()
            .map(|m| serde_json::to_value(m).unwrap_or(Value::Null))
            .unwrap_or(Value::Null);

        let manifest_rel = format!("{record_dir}/{MANIFEST_FILENAME}");
        let duration = item
            .duration
            .max(meta.as_ref().map(|m| m.duration).unwrap_or(0.0));
        let payload = SyncConfirmPayload {
            stream_id: stream.to_string(),
            start_time: item.start_time,
            duration,
            path: format!("recordings/{stream}/{manifest_rel}"),
            meta: meta_value,
        };

        self.authed(self.client.post(self.endpoint(api::path::record_sync())))
            .json(&payload)
            .send()
            .await?
            .error_for_status()
            .with_context(|| format!("confirm sync for {stream}/{record_dir}"))?;

        Ok(())
    }

    async fn read_meta(&self, stream: &str, record_dir: &str) -> Result<Option<RecordingMeta>> {
        let meta_path = format!("{stream}/{record_dir}/{META_FILENAME}");
        match self.operator.read(&meta_path).await {
            Ok(bytes) => {
                let meta: RecordingMeta = serde_json::from_slice(&bytes.to_vec())?;
                Ok(Some(meta))
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn load_index(&self, stream: &str) -> Result<Option<RecordingIndex>> {
        let index_path = format!("{stream}/{INDEX_FILENAME}");
        match self.operator.read(&index_path).await {
            Ok(bytes) => {
                let idx: RecordingIndex = serde_json::from_slice(&bytes.to_vec())?;
                Ok(Some(idx))
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn save_index(&self, stream: &str, index: &RecordingIndex) -> Result<()> {
        let index_path = format!("{stream}/{INDEX_FILENAME}");
        let bytes = serde_json::to_vec_pretty(index)?;
        self.operator.write(&index_path, bytes).await?;
        Ok(())
    }

    fn authed(&self, builder: RequestBuilder) -> RequestBuilder {
        if let Some(token) = &self.cfg.api_token {
            builder.bearer_auth(token)
        } else {
            builder
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

#[derive(Serialize)]
struct PresignUploadPayload {
    stream_id: String,
    filename: String,
    method: String,
}

#[derive(Deserialize)]
struct PresignUploadResponse {
    url: String,
    method: String,
    headers: HashMap<String, String>,
}

#[derive(Serialize)]
struct SyncConfirmPayload {
    stream_id: String,
    start_time: i64,
    duration: f64,
    path: String,
    meta: Value,
}

fn normalize_record_dir(path: &str) -> Result<String> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        Err(anyhow!("invalid record path: {path}"))
    } else {
        Ok(trimmed.to_string())
    }
}
