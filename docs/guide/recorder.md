# Recorder

The Recorder in liveion is an optional feature that automatically records live streams into MP4 fragments and saves them locally or to the cloud. The `recorder` feature must be enabled at compile time.

## Supported Codecs {#codec}

| container           | video codecs | audio codecs |
| ------------------- | ------------| ------------ |
| `Fragmented MP4`    | `H264`, `VP9`, `H265` | `Opus`       |

**Note:** Recorder does not support `VP8` codec because `VP8` requires a `WebM` container, which is not compatible with Fragmented MP4.

## Automatic Recording {#auto-recording}

When `auto_streams` patterns are configured, Live777 automatically starts recording when a matching stream goes **Up** (receives its first publisher) and stops when it goes **Down** (loses all publishers).

**Example:**
```toml
[recorder]
auto_streams = ["*"]              # Record all streams
# auto_streams = ["camera*", "room-*"]  # Record streams matching patterns
```

## Session Rotation {#rotation}

Recording sessions are automatically rotated to manage file size and storage efficiency. When a session accumulates duration equal to `max_recording_seconds`, Live777:

1. Closes the current set of media segments (`v_seg_*.m4s`, `a_seg_*.m4s`)
2. Finalizes the manifest (MPD) for the completed session
3. Creates a new timestamped directory with a fresh manifest
4. Continues recording in the new session

**Rotation behavior:**
- **Disabled**: Set `max_recording_seconds = 0` to record indefinitely without rotation
- **Default**: `max_recording_seconds = 86400` (24 hours)
- **Check interval**: Rotation checks run every ~10% of the max duration interval to minimize latency

This ensures recordings remain manageable while maintaining continuous capture.

Integrates with [Liveman](/guide/liveman) for centralized playback, recording metadata synchronization, and proxy access:

- **Manual Start**: Call `POST /api/record/:streamId` on a Live777 node returns the storage metadata (`record_id`, `record_dir`, and `mpd_path`)
- **Metadata Sync**: Live777 automatically syncs completed recording metadata to Liveman via `POST /api/record/sync`
- **Playback**: Liveman maintains a catalog of all recordings and proxies segment retrieval via `GET /api/record/object/{path}`
- **Status Tracking**: Query recording status and retrieve playback index through Liveman's unified API

### Recording ID

The `record_id` field is a 10-digit Unix timestamp (seconds) extracted from the output path:
- When using default paths: `record_id` is automatically populated with the session start timestamp
- When using custom `base_dir`: `record_id` is empty unless the path ends with a 10-digit timestamp
- Example: For path `camera01/1718200000/`, the `record_id` is `"1718200000"`

### Configuration

```toml
[recorder]
# Optional: Node alias to identify this Live777 instance in the cluster
node_alias = "live777-node-001"
```

::: tip
The `node_alias` is optional but recommended for multi-node deployments to help Liveman identify the source of recording metadata and correctly route requests to the appropriate Live777 node.
:::

## Configuration {#config}

Configure recording parameters in `live777.toml`:

```toml
[recorder]
# Stream name patterns for auto-recording, supports wildcards (default: [])
auto_streams = ["*"]              # Record all streams
# auto_streams = ["room1", "web-*"]  # Record specific streams

# Maximum duration (seconds) for a single recording session before rotation (default: 86_400)
max_recording_seconds = 86_400

# Optional: Node alias for multi-node deployments
node_alias = "live777-node-001"

# Storage backend configuration
[recorder.storage]
type = "fs"          # Storage type: "fs", "s3", or "oss"
root = "./storage"   # Root path for recordings (default: "./storage")
```

### Configuration Options

#### Basic Options

- `auto_streams`: Stream name patterns for auto-recording. Supports glob patterns with `*` and `?` wildcards (default: `[]`—no auto-recording)
  - Examples: `["*"]` (all streams), `["camera*", "room-*"]` (pattern-based)
- `max_recording_seconds`: Maximum cumulative duration (seconds) for a single recording session before automatic rotation (default: `86400` = 24 hours; set to `0` to disable auto-rotation)
- `node_alias`: Optional node identifier for multi-node deployments. Used by Liveman to identify and track recordings from this node (default: not set)

#### Storage Options

**File System (fs):**

- `type`: Must be `"fs"`
- `root`: Root directory path (default: `"./storage"`)

**S3 Backend:**

- `type`: Must be `"s3"`
- `bucket`: S3 bucket name (required)
- `root`: Root path within bucket (default: `"/"`)
- `region`: AWS region (optional, auto-detected from environment if not set)
- `endpoint`: Custom endpoint URL for S3-compatible services (optional)
- `access_key_id`: AWS access key ID (optional, can be loaded from environment)
- `secret_access_key`: AWS secret access key (optional, can be loaded from environment)
- `session_token`: Session token for temporary credentials (optional)
- `disable_config_load`: Set to `true` to disable automatic credential loading from environment/config files (default: `false`)
- `enable_virtual_host_style`: Enable virtual-hosted-style requests, e.g., `bucket.endpoint.com` instead of `endpoint.com/bucket` (default: `false`)

**OSS Backend:**

- `type`: Must be `"oss"`
- `bucket`: OSS bucket name (required)
- `root`: Root path within bucket (default: `"/"`)
- `region`: OSS region identifier, e.g., `"oss-cn-hangzhou"` (required)
- `endpoint`: OSS endpoint URL, e.g., `"https://oss-cn-hangzhou.aliyuncs.com"` (required)
- `access_key_id`: Alibaba Cloud access key ID (optional, can be loaded from environment)
- `access_key_secret`: Alibaba Cloud access key secret (optional, can be loaded from environment)
- `security_token`: Security token for STS temporary credentials (optional)

## Storage Backends {#storage}

### Local File System

```toml
[recorder.storage]
type = "fs"
root = "./storage"  # Or absolute path like "/var/lib/live777/recordings"
```

### AWS S3

Using IAM role (recommended for EC2/ECS):
```toml
[recorder.storage]
type = "s3"
bucket = "my-live777-bucket"
root = "/recordings"
region = "us-east-1"
```

Using explicit credentials:
```toml
[recorder.storage]
type = "s3"
bucket = "my-live777-bucket"
root = "/recordings"
region = "us-east-1"
access_key_id = "AKIA..."
secret_access_key = "..."
```

Using temporary credentials:
```toml
[recorder.storage]
type = "s3"
bucket = "my-live777-bucket"
root = "/recordings"
region = "us-east-1"
access_key_id = "ASIA..."
secret_access_key = "..."
session_token = "..."
```

### MinIO (S3-Compatible)

```toml
[recorder.storage]
type = "s3"
bucket = "live777-recordings"
root = "/recordings"
region = "us-east-1"
endpoint = "http://localhost:9000"
access_key_id = "minioadmin"
secret_access_key = "minioadmin"
enable_virtual_host_style = false
```

### Alibaba Cloud OSS

```toml
[recorder.storage]
type = "oss"
bucket = "my-oss-bucket"
root = "/recordings"
region = "oss-cn-hangzhou"
endpoint = "https://oss-cn-hangzhou.aliyuncs.com"
access_key_id = "your-access-key"
access_key_secret = "your-access-secret"
```

For STS temporary credentials:
```toml
[recorder.storage]
type = "oss"
bucket = "my-oss-bucket"
root = "/recordings"
region = "oss-cn-hangzhou"
endpoint = "https://oss-cn-hangzhou.aliyuncs.com"
access_key_id = "STS..."
access_key_secret = "..."
security_token = "..."
```

## API Reference {#api}

For detailed API documentation on recording control and playback, see:

- [Live777 API - Recorder](/guide/live777-api#recorder): Start, stop, and check recording status
- [Live777 API - VOD](/guide/live777-api#recorder): Direct access to recorded files and indexes
- [Liveman API - Recording & Playback](/guide/liveman-api#recording-playback): Centralized management for multi-node clusters

::: tip
For single-node deployments, use Live777's built-in recording and VOD APIs. For multi-node clusters, use Liveman's centralized APIs for unified access across all nodes.
:::

## MPD Path Conventions {#mpd}

### Default Paths

When `base_dir` is **not** provided:
- `record_dir`: `/:streamId/:record_id/` where `record_id` is a 10-digit Unix timestamp (seconds) at session start
- MPD location: `/:streamId/:record_id/manifest.mpd`

**Example:** `camera01/1718200000/manifest.mpd` (for a session starting 2024-06-12 19:00:00 UTC)

### Custom Paths

When `base_dir` **is** provided:
- `record_dir`: Matches the `base_dir` value exactly
- MPD location: `/{base_dir}/manifest.mpd`
- `record_id`: Empty string unless `base_dir` ends with a 10-digit timestamp

**Example:**
- Input: `base_dir = "recordings/mystream/2025-01-15"`
- Output: `record_dir = "recordings/mystream/2025-01-15"`, `record_id = ""`

### Rotation Paths

When a session reaches `max_recording_seconds`:
- The current session closes
- A new directory is created with a fresh timestamp: `/:streamId/:new_record_id/`
- No calendar-style directories are created automatically

**Example:** After rotation at session 2-hour mark:
```
stream1/
├── 1718200000/   (initial session, ~2 hours)
│   ├── manifest.mpd
│   ├── v_init.m4s
│   ├── v_seg_0001.m4s
│   └── ...
└── 1718207200/   (new session after rotation)
    ├── manifest.mpd
    └── ...
```

## File Structure {#file-structure}

Recorded files are organized hierarchically within the storage backend:

```
{storage_root}/
└── stream1/
    ├── 1762842203/                    # Session 1 (Unix timestamp)
    │   ├── manifest.mpd               # DASH manifest
    │   ├── v_init.m4s                 # Video initialization segment
    │   ├── a_init.m4s                 # Audio initialization segment
    │   ├── v_seg_0001.m4s             # Video segment 1
    │   ├── a_seg_0001.m4s             # Audio segment 1
    │   ├── v_seg_0002.m4s
    │   ├── a_seg_0002.m4s
    │   └── ...
    └── 1762845803/                    # Session 2 (after rotation)
        ├── manifest.mpd
        ├── v_init.m4s
        ├── a_init.m4s
        └── ...
```

### Segment Organization

- **Initialization Segments**: `v_init.m4s` and `a_init.m4s` contain codec initialization data
- **Media Segments**: `v_seg_NNNN.m4s` (video) and `a_seg_NNNN.m4s` (audio) are sequentially numbered
- **Manifest**: `manifest.mpd` is the DASH-compliant MPD file describing the playback timeline

### Storage Paths

- **Timestamp-based folders** (`stream/1762842203/`) are the canonical layout produced by Live777
- **Automatic rotations** maintain this layout; each new session gets its own timestamped directory
- **Custom `base_dir`**: If provided, overrides the default layout. Use only if you have specific organizational needs

### File Characteristics

- All segments are **Fragmented MP4** format (ISO/IEC 14496-12:2015)
- Each segment is independently seekable and decodable
- Typical segment duration: 1-10 seconds (configurable per codec)
- MPD manifest references all segments and their timing
