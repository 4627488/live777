# LiveVOD

LiveVOD 是一个轻量级回放服务，只依赖 `index.json`（JSONL）和 S3 存储配置。

## 配置

```toml
[http]
# listen = "0.0.0.0:8899"

# 回放索引路径（JSONL 或 JSON 数组）
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

- 列出流：`GET /api/playback`
- 列出流的录制会话：`GET /api/playback/{stream}`
- 时间点定位会话：`GET /api/playback/{stream}/at?ts=...`
  - `ts` 支持秒、毫秒或微秒。
- 代理对象：`GET /api/record/object/{path}`

当 `playback.signed_redirect = true` 时，非 MPD 文件将使用预签名 URL 重定向。
