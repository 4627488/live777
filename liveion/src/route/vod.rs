use crate::AppState;
use crate::error::AppError;
#[cfg(feature = "recorder")]
use crate::recorder::STORAGE;
use axum::extract::Path;
use axum::http::HeaderMap;
#[cfg(feature = "recorder")]
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router};

pub fn route() -> Router<AppState> {
    Router::new()
        .route("/{stream}/index", get(get_index))
        .route("/{stream}/{timestamp}/{filename}", get(get_file))
}

#[cfg(feature = "recorder")]
async fn get_index(Path(stream): Path<String>) -> crate::result::Result<Json<serde_json::Value>> {
    let op = {
        let guard = STORAGE.read().await;
        guard
            .clone()
            .ok_or(AppError::Throw("Storage not initialized".into()))?
    };

    let path = format!("{}/index.json", stream);
    match op.read(&path).await {
        Ok(bytes) => {
            let json: serde_json::Value = serde_json::from_slice(&bytes.to_vec())
                .map_err(|e| AppError::InternalServerError(anyhow::anyhow!(e)))?;
            Ok(Json(json))
        }
        Err(e) if e.kind() == opendal::ErrorKind::NotFound => {
            Ok(Json(serde_json::json!({ "items": [] })))
        }
        Err(e) => Err(AppError::InternalServerError(anyhow::anyhow!(e))),
    }
}

#[cfg(not(feature = "recorder"))]
async fn get_index(Path(_stream): Path<String>) -> crate::result::Result<Json<serde_json::Value>> {
    Err(AppError::Throw("feature recorder not enabled".into()))
}

#[cfg(feature = "recorder")]
async fn get_file(
    Path((stream, timestamp, filename)): Path<(String, String, String)>,
    _headers: HeaderMap,
) -> crate::result::Result<Response> {
    let op = {
        let guard = STORAGE.read().await;
        guard
            .clone()
            .ok_or(AppError::Throw("Storage not initialized".into()))?
    };

    let path = format!("{}/{}/{}", stream, timestamp, filename);

    match op.read(&path).await {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&filename).first_or_octet_stream();

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                // Enable CORS for VOD
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(axum::body::Body::from(bytes.to_bytes()))
                .unwrap())
        }
        Err(e) if e.kind() == opendal::ErrorKind::NotFound => Err(AppError::NotFound),
        Err(e) => Err(AppError::InternalServerError(anyhow::anyhow!(e))),
    }
}

#[cfg(not(feature = "recorder"))]
async fn get_file(
    Path((_stream, _timestamp, _filename)): Path<(String, String, String)>,
    _headers: HeaderMap,
) -> crate::result::Result<Response> {
    Err(AppError::Throw("feature recorder not enabled".into()))
}
