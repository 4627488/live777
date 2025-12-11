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

### Create a Stream

`POST` `/api/streams/:streamId`

`streamId` need unique id

Maybe you can use this configuration auto create stream

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

### Destroy a Stream

`DELETE` `/api/streams/:streamId`

Response: [204]

## Cascade

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

## Recorder

Recording and stream capture via Live777 node. **Requires `recorder` feature at compile time.**

### Start Recording a Stream

`POST` `/api/record/:streamId`

Starts recording the specified stream. The stream must be active (have a publisher) for recording to begin.

**Request Body (optional):**

```json
{
  "base_dir": "optional/custom/path"
}
```

Parameters:
- `base_dir` (optional): Override the storage path prefix. If omitted, Live777 uses `/:streamId/:record_id/` where `record_id` is the session start Unix timestamp. When a session reaches `max_recording_seconds` duration, a new timestamped directory is created automatically.

**Response:** `200 OK`

```json
{
  "id": "camera01",
  "record_id": "1718200000",
  "record_dir": "camera01/1718200000",
  "mpd_path": "camera01/1718200000/manifest.mpd"
}
```

Response Fields:
- `id`: The stream identifier (same as `:streamId`)
- `record_id`: 10-digit Unix timestamp extracted from the session path, or empty string if not inferable
- `record_dir`: The relative storage path where recordings are saved
- `mpd_path`: Absolute path to the DASH manifest file

### Recording Status

`GET` `/api/record/:streamId`

Check whether a stream is currently being recorded on this Live777 node.

**Response:** `200 OK`

```json
{
  "recording": true
}
```

Returns `true` if the stream is actively recording, `false` otherwise.

### Stop Recording

`DELETE` `/api/record/:streamId`

Stops an active recording session for the specified stream.

**Response:** `200 OK` (empty body)

Returns success regardless of whether a recording was active. This is idempotent and safe to call multiple times.

### Get Recording Index (VOD)

`GET` `/vod/:streamId/index`

Retrieves the recording index for a specific stream, listing all available recording sessions.

**Response:** `200 OK`

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

If no recordings exist for the stream, returns:
```json
{
  "items": []
}
```

### Get Recording File (VOD)

`GET` `/vod/:streamId/:timestamp/:filename`

Retrieves a specific recording file for playback or download.

**Path Parameters:**
- `streamId`: Stream identifier
- `timestamp`: Unix timestamp (10-digit) identifying the recording session
- `filename`: Name of the file to retrieve (e.g., `manifest.mpd`, `v_init.m4s`, `v_seg_0001.m4s`)

**Response:** `200 OK`

Returns the requested file with appropriate Content-Type header. The response includes CORS headers (`Access-Control-Allow-Origin: *`) for browser-based playback.

**Example:**
```
GET /vod/camera01/1718200000/manifest.mpd
```

Returns the DASH manifest for playback.

