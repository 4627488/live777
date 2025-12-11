# LiveMan HTTP API

Live777 Cluster Manager

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

Recording and playback related APIs for Liveman cluster (proxy, index listing, and recording management).

### List Streams with Recordings

`GET` `/api/playback`

Retrieve a list of all streams that have recording index entries in Liveman's database.

**Response:** `200 OK` `application/json`
```json
[
  "camera01",
  "roomA",
  "web-0001"
]
```

### List Recording Index by Stream

`GET` `/api/playback/{stream}`

Retrieve all recording sessions for a specific stream stored in Liveman.

**Response:** `200 OK` `application/json`
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

### Proxy Segment/Manifest Access

`GET` `/api/record/object/{path}`

Proxy access to recorded segment files and MPD manifests. Liveman retrieves files from the underlying storage backend and streams them to the client.

**Path parameter:** 
- `path`: URL-encoded storage path of the recorded object (e.g., `camera01/1718200000/manifest.mpd` or `camera01/1718200000/v_seg_0001.m4s`)

**Response:** `200 OK` - Binary media data

Content-Type inferred by file extension:
- `.mpd` → `application/dash+xml`
- `.m4s`, `.mp4` → `video/mp4` or `audio/mp4`
- Other → `application/octet-stream`

**Alternative Response:** `302 Found` redirect to storage backend presigned URL (when configured)

**Example - MPD Manifest:**
```
GET /api/record/object/camera01/1718200000/manifest.mpd
```
Response: `200 OK` `application/dash+xml`
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

### Start Recording (Manual)

`POST` `/api/record/{stream}`

Manually start recording a stream through Liveman. Liveman will:
1. Route the request to an available Live777 node (based on `node` query parameter or where the stream is currently published)
2. Forward the request to that node's recorder API
3. Store recording metadata in Liveman's database

**Query Parameters:**
- `node` (optional): Alias of specific Live777 node to route recording to. If omitted, uses the first node publishing this stream or the first available node.

**Request Body (optional):**
```json
{
  "base_dir": "optional/custom/path"
}
```

Parameters:
- `base_dir` (optional): Override storage path prefix on the target Live777 node

**Response:** `200 OK` `application/json`
```json
{
  "started": true,
  "mpd_path": "camera01/1718200000/manifest.mpd"
}
```

### Recording Status

`GET` `/api/record/{stream}`

Check if any node in the cluster is currently recording this stream.

**Response:** `200 OK` `application/json`
```json
{
  "recording": true
}
```

### Stop Recording

`DELETE` `/api/record/{stream}`

Stop recording of a stream on all nodes in the cluster that are recording it.

**Response:** `200 OK` `application/json`

### Presign Upload URL

`POST` `/api/storage/presign_upload`

Generate a presigned upload URL for direct storage backend uploads (used by Live777 nodes for optimized recording sync).

**Request Body:**
```json
{
  "stream_id": "camera01",
  "filename": "2025/07/24/v_seg_0001.m4s",
  "method": "PUT"
}
```

Parameters:
- `stream_id`: Stream identifier
- `filename`: Relative path within storage
- `method`: HTTP method (`PUT` for uploads)

**Response:** `200 OK` `application/json`
```json
{
  "url": "https://s3.region.amazonaws.com/bucket/camera01/2025/07/24/v_seg_0001.m4s?signature=...",
  "method": "PUT",
  "headers": {
    "Content-Type": "application/octet-stream"
  }
}
```

### Sync Recording Metadata

`POST` `/api/record/sync`

Synchronize recording metadata from a Live777 node to Liveman's database. **Typically called automatically by Live777 nodes; rarely needs manual invocation.**

**Request Body:**
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

Fields:
- `stream_id`: Stream name
- `start_time`: Session start Unix timestamp
- `duration`: Total session duration in seconds
- `path`: Relative storage path to MPD manifest
- `meta`: Recording metadata (codecs, resolution, size, etc.)

**Response:** `200 OK` `application/json`
```json
{
  "status": "ok"
}
```

## Node

`GET` `/api/nodes/`

Response: [200]

- `alias`: String, Alias must be unique
- `url`: String, Node API URL  
- `pub_max`: Int16, Maximum publish count
- `sub_max`: Int16, Maximum subscribe count
- `status`: StringEnum("running" | "stopped"), Node status

For Example:

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

### Get all Stream

**This API will merge all nodes streams**

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

For Example:

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

### Get a Stream Details

**This API will return a stream in all nodes**

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

