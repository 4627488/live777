#[cfg(feature = "recorder")]
use axum::http::StatusCode;
use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Json, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(not(feature = "recorder"))]
use crate::error::AppError;
use crate::{AppState, result::Result};

#[cfg_attr(not(feature = "recorder"), allow(dead_code))]
#[derive(Debug, Deserialize)]
struct PresignRequest {
    method: String,
    path: String,
    ttl_seconds: u64,
}

#[cfg_attr(not(feature = "recorder"), allow(dead_code))]
#[derive(Debug, Serialize)]
struct PresignResponse {
    url: String,
    headers: HashMap<String, String>,
}

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/api/storage/presign", post(presign))
        .route("/api/storage/ping", axum::routing::get(ping))
}

async fn ping(State(state): State<AppState>) -> Result<Response> {
    #[cfg(feature = "recorder")]
    {
        if state.file_storage.is_some() {
            Ok((StatusCode::OK, "ok").into_response())
        } else {
            Ok((StatusCode::SERVICE_UNAVAILABLE, "storage not configured").into_response())
        }
    }

    #[cfg(not(feature = "recorder"))]
    {
        let _ = state;
        Err(AppError::InternalServerError(anyhow::anyhow!(
            "feature recorder not enabled",
        )))
    }
}

async fn presign(
    State(state): State<AppState>,
    Json(req): Json<PresignRequest>,
) -> Result<Response> {
    #[cfg(feature = "recorder")]
    {
        let Some(ref operator) = state.file_storage else {
            return Ok((StatusCode::SERVICE_UNAVAILABLE, "storage not configured").into_response());
        };

        let ttl = std::time::Duration::from_secs(req.ttl_seconds.max(30));
        let result = match req.method.as_str() {
            "GET" => operator.presign_read(&req.path, ttl).await,
            "PUT" => operator.presign_write(&req.path, ttl).await,
            _ => {
                return Ok((StatusCode::BAD_REQUEST, "unsupported method").into_response());
            }
        };

        match result {
            Ok(presigned) => {
                let mut headers = HashMap::new();
                for (name, value) in presigned.header() {
                    headers.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
                }
                let body = PresignResponse {
                    url: presigned.uri().to_string(),
                    headers,
                };
                Ok(Json(body).into_response())
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("presign failed: {e}"),
            )
                .into_response()),
        }
    }

    #[cfg(not(feature = "recorder"))]
    {
        let _ = state;
        let _ = req;
        Err(AppError::InternalServerError(anyhow::anyhow!(
            "feature recorder not enabled",
        )))
    }
}
