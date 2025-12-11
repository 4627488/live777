# LiveMan HTTP API

Live777 集群管理器

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

## Recording & Playback

录制与回放相关 API（用于 Liveman 集群的代理、索引列表和录制管理）。

### 列出存在录制索引的流

`GET` `/api/playback`

检索 Liveman 数据库中具有录制索引条目的所有流列表。

**响应：** `200 OK` `application/json`
```json
[
  "camera01",
  "roomA",
  "web-0001"
]
```

### 按流列出录制索引

`GET` `/api/playback/{stream}`

检索指定流在 Liveman 中存储的所有录制会话。

**响应：** `200 OK` `application/json`
```json
[
  {
    "record": "camera01/1718200000",
    "mpd_path": "camera01/1718200000/manifest.mpd",
    "start_time": 1718200000,
    "duration": 3600.0,
    "meta": {
      "video_codec": "avc1.64001f",
      "audio_codec": "opus",
      "width": 1920,
      "height": 1080,
      "size": 1234567890
    }
  }
]
```

### 代理分片/清单访问

`GET` `/api/record/object/{path}`

代理访问录制的分片文件和 MPD 清单。Liveman 从底层存储后端检索文件并流式传输给客户端。

**路径参数：**
- `path`：URL 编码的录制对象存储路径（例如 `camera01/1718200000/manifest.mpd` 或 `camera01/1718200000/v_seg_0001.m4s`）

**响应：** `200 OK` - 二进制媒体数据

Content-Type 根据文件扩展名推断：
- `.mpd` → `application/dash+xml`
- `.m4s`、`.mp4` → `video/mp4` 或 `audio/mp4`
- 其他 → `application/octet-stream`

**替代响应：** `302 Found` 重定向到存储后端预签名 URL（若配置）

**示例 - MPD 清单：**
```
GET /api/record/object/camera01/1718200000/manifest.mpd
```
响应：`200 OK` `application/dash+xml`
```xml
<?xml version="1.0"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Period>
    <AdaptationSet>
      <Representation id="video">
        <SegmentList>
          <SegmentURL media="v_init.m4s"/>
          <SegmentURL media="v_seg_0001.m4s"/>
          <SegmentURL media="v_seg_0002.m4s"/>
        </SegmentList>
      </Representation>
    </AdaptationSet>
  </Period>
</MPD>
```

### 启动录制（手动）

`POST` `/api/record/{stream}`

通过 Liveman 手动启动流的录制。Liveman 将：
1. 将请求路由到可用的 Live777 节点（基于 `node` 查询参数或流当前发布的节点）
2. 将请求转发到该节点的录制 API
3. 在 Liveman 的数据库中存储录制元数据

**查询参数：**
- `node`（可选）：特定 Live777 节点的别名以路由录制请求。如果省略，使用首先发布此流的节点或第一个可用节点。

**请求体（可选）：**
```json
{
  "base_dir": "optional/custom/path"
}
```

参数说明：
- `base_dir`（可选）：覆盖目标 Live777 节点上的存储路径前缀

**响应：** `200 OK` `application/json`
```json
{
  "started": true,
  "mpd_path": "camera01/1718200000/manifest.mpd"
}
```

### 录制状态

`GET` `/api/record/{stream}`

检查集群中的任何节点是否正在录制该流。

**响应：** `200 OK` `application/json`
```json
{
  "recording": true
}
```

### 停止录制

`DELETE` `/api/record/{stream}`

停止集群中所有正在录制此流的节点的录制。

**响应：** `200 OK` `application/json`

### 预签名上传 URL

`POST` `/api/storage/presign_upload`

生成预签名上传 URL 用于直接上传到存储后端（由 Live777 节点用于优化的录制同步）。

**请求体：**
```json
{
  "stream_id": "camera01",
  "filename": "2025/07/24/v_seg_0001.m4s",
  "method": "PUT"
}
```

参数说明：
- `stream_id`：流标识符
- `filename`：存储内的相对路径
- `method`：HTTP 方法（`PUT` 用于上传）

**响应：** `200 OK` `application/json`
```json
{
  "url": "https://s3.region.amazonaws.com/bucket/camera01/2025/07/24/v_seg_0001.m4s?signature=...",
  "method": "PUT",
  "headers": {
    "Content-Type": "application/octet-stream"
  }
}
```

### 同步录制元数据

`POST` `/api/record/sync`

将录制元数据从 Live777 节点同步到 Liveman 的数据库。**通常由 Live777 节点自动调用；很少需要手动调用。**

**请求体：**
```json
{
  "stream_id": "camera01",
  "start_time": 1718200000,
  "duration": 3600.0,
  "path": "camera01/1718200000/manifest.mpd",
  "meta": {
    "start_time": 1718200000,
    "end_time": 1718203600,
    "duration": 3600.0,
    "size": 1234567890,
    "video_codec": "avc1.64001f",
    "audio_codec": "opus",
    "width": 1920,
    "height": 1080
  }
}
```

字段说明：
- `stream_id`：流名称
- `start_time`：会话启动 Unix 时间戳
- `duration`：总会话时长（秒）
- `path`：MPD 清单的相对存储路径
- `meta`：录制元数据（编码、分辨率、大小等）

**响应：** `200 OK` `application/json`
```json
{
  "status": "ok"
}
```

## Node

`GET` `/api/nodes/`

Response: [200]

- `alias`: String, 别名必须唯一
- `url`: String, 节点 API 的 URL 地址
- `pub_max`: Int16, 最大支持推流数
- `sub_max`: Int16, 最大支持订阅数
- `status`: StringEnum("running" | "stopped"), 节点状态

例如:

```json
[
  {
    "alias": "buildin-0",
    "url": "http://127.0.0.1:55581",
    "pub_max": 65535,
    "sub_max": 1,
    "status": "running"
  },
  {
    "alias": "buildin-1",
    "url": "http://127.0.0.1:55582",
    "pub_max": 65535,
    "sub_max": 1,
    "status": "running"
  },
  {
    "alias": "buildin-2",
    "url": "http://127.0.0.1:55583",
    "pub_max": 65535,
    "sub_max": 1,
    "status": "running"
  }
]
```

## Stream

### 获取所有流

**此API将合并所有节点的流**

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
- `(publish | subscribe).sessions.[].cascade`: Optional(Object(Cascade)

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
            "src": "http://localhost:7777/whep/web-0",
            "resource": "http://localhost:7777/session/web-0/aabc02240abfc7f4800e8d9a6f087808"
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
            "dst": "http://localhost:7777/whip/push",
            "resource": "http://localhost:7777/session/push/08c1f2a0a60b0deeb66ee572bd369f80"
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

### 获取流详情

**此API将返回指定流在所有节点上的信息**

`GET` `/api/streams/:streamId`

Response: [200]

```json
{
  "buildin-1": {
    "id": "web-0",
    "createdAt": 1719415906241,
    "publish": {
      "leaveAt": 0,
      "sessions": []
    },
    "subscribe": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "04eaae154975b61d62fc2e81b2b0862f",
          "createdAt": 1719415906274,
          "state": "connected"
        }
      ]
    }
  },
  "buildin-0": {
    "id": "web-0",
    "createdAt": 1719415876416,
    "publish": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "6ea2c116b93dde47032c7ea19349dc78",
          "createdAt": 1719415876510,
          "state": "connected"
        }
      ]
    },
    "subscribe": {
      "leaveAt": 0,
      "sessions": [
        {
          "id": "369227db507bf2addbb55313e0eb99a0",
          "createdAt": 1719415885569,
          "state": "connected"
        }
      ]
    }
  }
}
```

