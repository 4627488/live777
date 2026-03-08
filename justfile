#!/usr/bin/env -S just --justfile

host := "127.0.0.1"
port := "7777"
server := "http://" + host + ":" + port
stream := "test-stream"

isdp := "i.sdp"
osdp := "o.sdp"

asrc := "-f lavfi -i sine=frequency=1000"
vsrc := "-f lavfi -i testsrc=size=640x480:rate=30"

h264 := "libx264 -preset ultrafast -tune zerolatency -profile:v baseline -level 3.0 -pix_fmt yuv420p -g 30 -keyint_min 30 -b:v 1000k -minrate 1000k -maxrate 1000k -bufsize 1000k"
h265 := "libx265 -preset ultrafast -tune zerolatency -x265-params keyint=30:min-keyint=30:bframes=0:repeat-headers=1 -pix_fmt yuv420p -b:v 1000k -minrate 1000k -maxrate 1000k -bufsize 1000k"
vp9  := "libvpx-vp9 -pix_fmt yuv420p"

default:
    just --list

build:
    pnpm install
    pnpm run build
    cargo build --release --all-targets --all-features

docs:
    pnpm run docs:dev

run:
    cargo run --features=webui

run-cluster:
    cargo run --bin=livenil --features=webui -- -c conf/livenil

[group('simple-rtp')]
mpeg-rtp-h264:
    cargo run --bin=whipinto -- -i {{isdp}} -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{vsrc}} -vcodec {{h264}} -f rtp 'rtp://{{host}}:5002' -sdp_file {{isdp}}"

[group('simple-rtp')]
mpeg-rtp-h265:
    cargo run --bin=whipinto -- -i {{isdp}} -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{vsrc}} -vcodec {{h265}} -f rtp 'rtp://{{host}}:5002' -sdp_file {{isdp}}"

[group('simple-rtp')]
mpeg-rtp-vp8:
    cargo run --bin=whipinto -- -i {{isdp}} -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{vsrc}} -vcodec libvpx -f rtp rtp://{{host}}:5002 -sdp_file {{isdp}}"

# 4K (3840×2160)
[group('simple-rtp')]
mpeg-rtp-4k:
    cargo run --bin=whipinto -- -i {{isdp}} -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re -f lavfi -i testsrc=size=3840x2160:rate=30 -strict experimental -vcodec {{vp9}} -f rtp rtp://{{host}}:5002 -sdp_file {{isdp}}"

[group('simple-rtp')]
play-rtp:
    cargo run --bin=whepfrom -- -o "rtp://localhost?video=9000&audio=9002" --sdp-file {{osdp}} -w {{server}}/whep/{{stream}} --command \
        "ffplay -protocol_whitelist rtp,file,udp -i {{osdp}}"


# Aa rtsp server receive stream
[group('simple-rtsp')]
mpeg-rtsp:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{asrc}} {{vsrc}} -acodec libopus -vcodec libvpx -f rtsp rtsp://{{host}}:8550"

[group('simple-rtsp')]
mpeg-rtsp-tcp:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{asrc}} {{vsrc}} -acodec libopus -vcodec libvpx -rtsp_transport tcp -f rtsp rtsp://{{host}}:8550"

[group('simple-rtsp')]
mpeg-rtsp-vp9:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{asrc}} {{vsrc}} -acodec libopus -strict experimental -vcodec {{vp9}} -f rtsp rtsp://{{host}}:8550"

[group('simple-rtsp')]
mpeg-rtsp-h264:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{vsrc}} -vcodec {{h264}} -f rtsp rtsp://{{host}}:8550"

mpeg-rtsp-h264-raw:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{vsrc}} -vcodec libx264 -f rtsp rtsp://{{host}}:8550"

[group('simple-rtsp')]
mpeg-rtsp-h265:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/{{stream}} --command \
        "ffmpeg -re {{vsrc}} -vcodec {{h265}} -f rtsp rtsp://{{host}}:8550"

[group('simple-rtsp')]
play-rtsp:
    cargo run --bin=whepfrom -- -o rtsp-listen://{{host}}:8650 -w {{server}}/whep/{{stream}} --command \
        "ffplay rtsp://{{host}}:8650"

[group('simple-rtsp')]
play-rtsp-tcp:
    cargo run --bin=whepfrom -- -o rtsp-listen://{{host}}:8650 -w {{server}}/whep/{{stream}} --command \
        "ffplay rtsp://{{host}}:8650 -rtsp_transport tcp"


[group('cycle-rtsp')]
cycle-rtsp-0a:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8550 -w {{server}}/whip/cycle-rtsp-a --command \
        "ffmpeg -re {{asrc}} {{vsrc}} -acodec libopus -vcodec libvpx -f rtsp rtsp://{{host}}:8550"

[group('cycle-rtsp')]
cycle-rtsp-1a:
    cargo run --bin=whepfrom -- -o rtsp-listen://{{host}}:8650 -w {{server}}/whep/cycle-rtsp-a

[group('cycle-rtsp')]
cycle-rtsp-2b:
    cargo run --bin=whipinto -- -i rtsp://{{host}}:8650 -w {{server}}/whip/cycle-rtsp-b

[group('cycle-rtsp')]
cycle-rtsp-3c:
    cargo run --bin=whipinto -- -i rtsp-listen://{{host}}:8750 -w {{server}}/whip/cycle-rtsp-c

[group('cycle-rtsp')]
cycle-rtsp-4b:
    cargo run --bin=whepfrom -- -o rtsp://{{host}}:8750 -w {{server}}/whep/cycle-rtsp-b

[group('cycle-rtsp')]
cycle-rtsp-5c:
    cargo run --bin=whepfrom -- -o rtsp-listen://{{host}}:8850 -w {{server}}/whep/cycle-rtsp-c --command \
        "ffplay rtsp://{{host}}:8850"

