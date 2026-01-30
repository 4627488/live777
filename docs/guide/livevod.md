# LiveVOD

LiveVOD is a lightweight playback service that only depends on `index.json` (JSONL) and S3 storage configuration.

## Configuration

```toml
[http]
# listen = "0.0.0.0:8899"

# Playback index path (JSONL or JSON array)
index_path = "./recordings/index.json"

[storage]
type = "s3"
bucket = "my-live777-bucket"
root = "/recordings"
region = "us-east-1"

[playback]
# signed_redirect = false
# signed_ttl_seconds = 60
```

## APIs

- List streams: `GET /api/playback`
- List records for stream: `GET /api/playback/{stream}`
- Find record by timestamp: `GET /api/playback/{stream}/at?ts=...`
  - `ts` accepts seconds, milliseconds, or microseconds.
- Proxy object: `GET /api/record/object/{path}`

When `playback.signed_redirect = true`, non-MPD objects are redirected using presigned URLs.
