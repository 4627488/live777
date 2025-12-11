# Recorder

liveion 的 Recorder 是一个可选功能，用于将实时流自动录制为 MP4 分片并保存到本地或云。需要在编译时启用 `recorder` 特性。

## 目前支持的编码 {#codec}

| 容器  | 视频编码                | 音频编码   |
| -------- | --------------------------- | -------------- |
| `Fragmented MP4`    | `H264`, `VP9`, `H265`| `Opus`       |

**注意**：Recorder 暂不支持 `VP8` 编码，因为 `VP8` 需要 `WebM` 容器，与 Fragmented MP4 不兼容。

## 自动录制 {#auto-recording}

当配置了 `auto_streams` 模式时，Live777 会在匹配的流**上线**（首次收到发布者）时自动开始录制，在流**下线**（失去所有发布者）时停止录制。

**示例：**
```toml
[recorder]
auto_streams = ["*"]              # 录制所有流
# auto_streams = ["camera*", "room-*"]  # 录制匹配模式的流
```

## 会话轮转 {#rotation}

为了管理文件大小和存储效率，Live777 会自动轮转录制会话。当一个会话的累计时长达到 `max_recording_seconds` 时：

1. 关闭当前的媒体分片（`v_seg_*.m4s`、`a_seg_*.m4s`）
2. 完成当前会话的 MPD 清单
3. 使用新的时间戳目录创建新会话
4. 继续在新会话中录制

**轮转行为：**
- **禁用**：设置 `max_recording_seconds = 0` 可无限期录制而不轮转
- **默认值**：`max_recording_seconds = 86400`（24 小时）
- **检查间隔**：轮转检查每隔约最大时长的 10% 运行一次，以最小化延迟

这确保了录制文件保持可管理的大小，同时保持持续的捕获能力。

## Liveman 集成 {#liveman}

与 [Liveman](/zh/guide/liveman) 集成以实现集中式回放、录制元数据同步和代理访问：

- **手动启动**：调用 Live777 节点上的 `POST /api/record/:streamId` 返回存储元数据（`record_id`、`record_dir`、`mpd_path`）
- **元数据同步**：Live777 自动通过 `POST /api/record/sync` 将完成的录制元数据同步到 Liveman
- **回放**：Liveman 维护所有录制的目录，并通过 `GET /api/record/object/{path}` 代理分片获取
- **状态追踪**：通过 Liveman 的统一 API 查询录制状态和检索回放索引

### 录制 ID

`record_id` 字段是从输出路径中提取的 10 位 Unix 时间戳（秒）：
- 使用默认路径时：`record_id` 自动填充为会话启动时的时间戳
- 使用自定义 `base_dir` 时：除非路径以 10 位时间戳结尾，否则 `record_id` 为空
- 示例：对于路径 `camera01/1718200000/`，`record_id` 为 `"1718200000"`

### 配置

```toml
[recorder]
# 可选：节点别名，用于在集群中标识此 Live777 实例
node_alias = "live777-node-001"
```

::: tip 注意
`node_alias` 是可选的，但在多节点部署中建议配置，以帮助 Liveman 识别录制元数据的来源，并正确路由请求到相应的 Live777 节点。
:::

## 配置说明 {#config}

在 `live777.toml` 中配置录制参数：

```toml
[recorder]
# 自动录制的流名称模式，支持通配符（默认：空列表）
auto_streams = ["*"]                # 录制所有流
# auto_streams = ["room1", "web-*"]   # 仅录制指定流

# 单个录制会话的最大持续时间（秒），超过即重新开一个录制（默认：86_400）
max_recording_seconds = 86_400

# 可选：多节点部署的节点别名
node_alias = "live777-node-001"

# 存储后端配置
[recorder.storage]
type = "fs"        # 存储类型: "fs"、"s3" 或 "oss"
root = "./storage" # 录制文件根路径（默认："./storage"）
```

### 配置选项

#### 基础选项

- `auto_streams`: 自动录制的流名称模式。支持 `*` 和 `?` 通配符的通配模式（默认：`[]`—不自动录制）
  - 示例：`["*"]`（所有流），`["camera*", "room-*"]`（基于模式）
- `max_recording_seconds`: 单个录制会话的最大累计时长（秒），超过即自动轮转（默认：`86400` = 24 小时；设为 `0` 禁用自动轮转）
- `node_alias`: 可选的节点标识符，用于多节点部署。由 Liveman 用来识别和追踪来自此节点的录制（默认：不设置）

#### 存储选项

**文件系统 (fs)：**

- `type`: 必须为 `"fs"`
- `root`: 根目录路径（默认：`"./storage"`）

**S3 后端：**

- `type`: 必须为 `"s3"`
- `bucket`: S3 存储桶名称（必需）
- `root`: 存储桶内的根路径（默认：`"/"`）
- `region`: AWS 区域（可选，未设置时从环境自动检测）
- `endpoint`: S3 兼容服务的自定义端点 URL（可选）
- `access_key_id`: AWS 访问密钥 ID（可选，可从环境加载）
- `secret_access_key`: AWS 访问密钥 Secret（可选，可从环境加载）
- `session_token`: 临时凭证的会话令牌（可选）
- `disable_config_load`: 设为 `true` 禁用从环境/配置文件自动加载凭证（默认：`false`）
- `enable_virtual_host_style`: 启用虚拟主机样式请求，如 `bucket.endpoint.com` 而非 `endpoint.com/bucket`（默认：`false`）

**OSS 后端：**

- `type`: 必须为 `"oss"`
- `bucket`: OSS 存储桶名称（必需）
- `root`: 存储桶内的根路径（默认：`"/"`）
- `region`: OSS 区域标识符，如 `"oss-cn-hangzhou"`（必需）
- `endpoint`: OSS 端点 URL，如 `"https://oss-cn-hangzhou.aliyuncs.com"`（必需）
- `access_key_id`: 阿里云访问密钥 ID（可选，可从环境加载）
- `access_key_secret`: 阿里云访问密钥 Secret（可选，可从环境加载）
- `security_token`: STS 临时凭证的安全令牌（可选）

## 存储后端 {#storage}

### 本地文件系统

```toml
[recorder.storage]
type = "fs"
root = "./storage"  # 或使用绝对路径如 "/var/lib/live777/recordings"
```

### AWS S3

使用 IAM 角色（推荐用于 EC2/ECS）：
```toml
[recorder.storage]
type = "s3"
bucket = "my-live777-bucket"
root = "/recordings"
region = "us-east-1"
```

使用显式凭证：
```toml
[recorder.storage]
type = "s3"
bucket = "my-live777-bucket"
root = "/recordings"
region = "us-east-1"
access_key_id = "AKIA..."
secret_access_key = "..."
```

使用临时凭证：
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

### MinIO（S3兼容）

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

### 阿里云 OSS

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

使用 STS 临时凭证：
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

## API 参考 {#api}

有关录制控制和播放的详细 API 文档，请参阅：

- [Live777 API - 录制](/zh/guide/live777-api#recorder)：启动、停止和检查录制状态
- [Live777 API - VOD](/zh/guide/live777-api#recorder)：直接访问录制文件和索引
- [Liveman API - 录制与回放](/zh/guide/liveman-api#recording-playback)：多节点集群的集中管理

::: tip 提示
单节点部署使用 Live777 的内置录制和 VOD API。多节点集群使用 Liveman 的集中式 API，以实现跨所有节点的统一访问。
:::

## MPD 路径规则 {#mpd}

### 默认路径

当**未**提供 `base_dir` 时：
- `record_dir`：`/:streamId/:record_id/`，其中 `record_id` 是会话启动时的 10 位 Unix 时间戳
- MPD 位置：`/:streamId/:record_id/manifest.mpd`

**示例：** `camera01/1718200000/manifest.mpd`（2024-06-12 19:00:00 UTC 启动的会话）

### 自定义路径

当**已**提供 `base_dir` 时：
- `record_dir`：与 `base_dir` 值完全一致
- MPD 位置：`/{base_dir}/manifest.mpd`
- `record_id`：除非 `base_dir` 以 10 位时间戳结尾，否则为空字符串

**示例：**
- 输入：`base_dir = "recordings/mystream/2025-01-15"`
- 输出：`record_dir = "recordings/mystream/2025-01-15"`，`record_id = ""`

### 轮转路径

当会话达到 `max_recording_seconds` 时：
- 当前会话关闭
- 使用新时间戳创建新目录：`/:streamId/:new_record_id/`
- 不会自动创建日历风格的目录

**示例：** 在会话 2 小时标记时轮转：
```
stream1/
├── 1718200000/   （初始会话，约 2 小时）
│   ├── manifest.mpd
│   ├── v_init.m4s
│   ├── v_seg_0001.m4s
│   └── ...
└── 1718207200/   （轮转后的新会话）
    ├── manifest.mpd
    └── ...
```

## 文件组织结构 {#file-structure}

录制文件在存储后端中按层级组织：

```
{storage_root}/
└── stream1/
    ├── 1762842203/                    # 会话 1（Unix 时间戳）
    │   ├── manifest.mpd               # DASH 清单
    │   ├── v_init.m4s                 # 视频初始化段
    │   ├── a_init.m4s                 # 音频初始化段
    │   ├── v_seg_0001.m4s             # 视频段 1
    │   ├── a_seg_0001.m4s             # 音频段 1
    │   ├── v_seg_0002.m4s
    │   ├── a_seg_0002.m4s
    │   └── ...
    └── 1762845803/                    # 会话 2（轮转后）
        ├── manifest.mpd
        ├── v_init.m4s
        ├── a_init.m4s
        └── ...
```

### 分片组织

- **初始化段**：`v_init.m4s` 和 `a_init.m4s` 包含编码初始化数据
- **媒体段**：`v_seg_NNNN.m4s`（视频）和 `a_seg_NNNN.m4s`（音频）按顺序编号
- **清单**：`manifest.mpd` 是符合 DASH 标准的 MPD 文件，描述回放时间线

### 存储路径

- **基于时间戳的目录**（`stream/1762842203/`）是 Live777 生成的规范布局
- **自动轮转**维持此布局；每个新会话都获得自己的时间戳目录
- **自定义 `base_dir`**：如果提供，覆盖默认布局。仅在有特定组织需求时使用

### 文件特性

- 所有分片都是 **Fragmented MP4** 格式（ISO/IEC 14496-12:2015）
- 每个分片都可独立寻址和解码
- 典型分片时长：1-10 秒（每个编码可配置）
- MPD 清单引用所有分片及其时序
