# Live777 HTTP API

## WHIP && WHEP

`POST` `/whip/:streamId`

Response: [201]

`POST` `/whep/:streamId`

Response: [201]

* * *

`PATCH` `/session/:streamId/:sessionId`

Response: [204]

`DELETE` `/session/:streamId/:sessionId`

Response: [204]

## Stream

### 创建一个流

`POST` `/api/streams/:streamId`

`streamId` 需要唯一标识符​​

你可以使用此配置自动创建流​​

```toml
[strategy]
# WHIP auto a stream
auto_create_whip = true
# WHEP auto a stream
auto_create_whep = true
```

Response: [204]

### Get all Stream

`GET` `/api/streams/`

Response: [200]

- `id`: String, `streamId`
- `createdAt`: Int, `timestamp`
- `publish`: `Object(PubSub)`, about publisher
- `subscribe`: `Object(PubSub)`, about subscriber
- `(publish | subscribe).leaveAt`: Int, `timestamp`
- `(publish | subscribe).sessions`: Array, `sessions`
- `(publish | subscribe).sessions.[].id`: String, `sessionId`
- `(publish | subscribe).sessions.[].createdAt`: Int, `timestamp`
- `(publish | subscribe).sessions.[].state`: String, [RTCPeerConnection/connectionState](https://developer.mozilla.org/en-US/docs/Web/API/RTCPeerConnection/connectionState#value)
- `(publish | subscribe).sessions.[].cascade`: Optional(Object(Cascade))
- `(publish | subscribe).sessions.[].cascade.sourceUrl`: Optional(String(URL))
- `(publish | subscribe).sessions.[].cascade.targetUrl`: Optional(String(URL))
- `(publish | subscribe).sessions.[].cascade.sessionUrl`: String(URL)

例如:

```json
[
  {
    "id": "push",
    "createdAt": 1719326206862,
    "publish": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "08c1f2a0a60b0deeb66ee572bd369f80",
          "createdAt": 1719326206947,
          "state": "connected"
        }
      ]
    },
    "subscribe": {
      "leaveAt": 1719326206862,
      "sessions": []
    }
  },
  {
    "id": "pull",
    "createdAt": 1719326203854,
    "publish": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "41b2c52da4fb1eed5a3bff9a9a200d80",
          "createdAt": 1719326205079,
          "state": "connected",
          "cascade": {
            "sourceUrl": "http://localhost:7777/whep/web-0",
            "sessionUrl": "http://localhost:7777/session/web-0/aabc02240abfc7f4800e8d9a6f087808"
          }
        }
      ]
    },
    "subscribe": {
      "leaveAt": 1719326203854,
      "sessions": []
    }
  },
  {
    "id": "web-0",
    "createdAt": 1719326195910,
    "publish": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "0dc47d8da8eb0a64fe40f461f47c2a36",
          "createdAt": 1719326196264,
          "state": "connected"
        }
      ]
    },
    "subscribe": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "aabc02240abfc7f4800e8d9a6f087808",
          "createdAt": 1719326204997,
          "state": "connected"
        },
        {
          "id": "dab1a9e88b2400cfd4bcfb4487588ef3",
          "createdAt": 1719326206798,
          "state": "connected",
          "cascade": {
            "targetUrl": "http://localhost:7777/whip/push",
            "sessionUrl": "http://localhost:7777/session/push/08c1f2a0a60b0deeb66ee572bd369f80"
          }
        },
        {
          "id": "685beee8650b761116b581a4a87ca9b9",
          "createdAt": 1719326228314,
          "state": "connected"
        }
      ]
    }
  }
]
```

### 销毁一个流

`DELETE` `/api/streams/:streamId`

Response: [204]

## ​级联

`POST` `/api/cascade/:streamId`

Request:

```json
{
  "token": "",
  "sourceUrl": "",
  "targetUrl": "",
}
```

- `token`: Option, auth header
- `sourceUrl`: `Option<WHEP url>`. if has, use pull mode
- `targetUrl`: `Option<WHIP url>`. if has, use push mode
- `sourceUrl` and `targetUrl` at the same time can only one

## 录制

Live777 节点的录制和流捕获功能。**需要在编译时启用 `recorder` 特性。**

### 开始录制流

`POST` `/api/record/:streamId`

开始录制指定的流。流必须处于活跃状态（有发布者）才能开始录制。

**请求体（可选）：**

```json
{
  "base_dir": "optional/custom/path"
}
```

参数说明：
- `base_dir`（可选）：覆盖默认的存储路径前缀。如果不设置，Live777 使用 `/:streamId/:record_id/`，其中 `record_id` 是会话启动时的 Unix 时间戳。当会话时长达到 `max_recording_seconds` 时，会自动创建新的时间戳目录。

**响应：** `200 OK`

```json
{
  "id": "camera01",
  "record_id": "1718200000",
  "record_dir": "camera01/1718200000",
  "mpd_path": "camera01/1718200000/manifest.mpd"
}
```

响应字段说明：
- `id`：流标识符（与 `:streamId` 相同）
- `record_id`：从会话路径中提取的 10 位 Unix 时间戳，如无法推断则为空字符串
- `record_dir`：录制文件存储的相对路径
- `mpd_path`：DASH 清单文件的绝对路径

### 录制状态

`GET` `/api/record/:streamId`

检查此 Live777 节点上是否正在录制该流。

**响应：** `200 OK`

```json
{
  "recording": true
}
```

流正在录制时返回 `true`，否则返回 `false`。

### 停止录制

`DELETE` `/api/record/:streamId`

停止指定流的录制会话。

**响应：** `200 OK`（空响应体）

无论是否有活动录制，都返回成功。该操作是幂等的，可以安全地多次调用。
### 获取录制索引 (VOD)

`GET` `/vod/:streamId/index`

检索特定流的录制索引，列出所有可用的录制会话。

**响应：** `200 OK`

```json
{
  "items": [
    {
      "stream_id": "camera01",
      "start_time": 1718200000,
      "duration": 3600.0,
      "path": "1718200000/",
      "status": "LocalSaved"
    }
  ]
}
```

如果该流没有录制，返回：
```json
{
  "items": []
}
```

### 获取录制文件 (VOD)

`GET` `/vod/:streamId/:timestamp/:filename`

检索特定的录制文件用于播放或下载。

**路径参数：**
- `streamId`：流标识符
- `timestamp`：Unix 时间戳（10 位数字），标识录制会话
- `filename`：要检索的文件名（例如 `manifest.mpd`、`v_init.m4s`、`v_seg_0001.m4s`）

**响应：** `200 OK`

返回请求的文件，带有适当的 Content-Type 头。响应包含 CORS 头（`Access-Control-Allow-Origin: *`）以支持浏览器播放。

**示例：**
```
GET /vod/camera01/1718200000/manifest.mpd
```

返回用于播放的 DASH 清单。