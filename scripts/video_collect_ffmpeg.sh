#!/bin/bash
# scripts/video_collect_ffmpeg.sh — 校验 Direct 版的 FFmpeg 视频资源。
#
# 用法: video_collect_ffmpeg.sh [DEST_DIR]（默认 src-tauri/resources/video）
#
# 资源齐且可执行（ffmpeg/ffprobe 真跑一次）就放行；缺失或跑不起来时
# 调 scripts/collect_macos_tools.sh 重新收集。收集来源是本机 Homebrew，
# 其 GPL 配置在发布前必须过许可审查（见 THIRD_PARTY_NOTICES.md）。

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${1:-$PROJECT_DIR/src-tauri/resources/video}"

run_ok() {
  local binary="$1"
  [ -x "$binary" ] || return 1
  "$binary" -version >/dev/null 2>&1
}

if run_ok "$DEST/bin/ffmpeg" && run_ok "$DEST/bin/ffprobe"; then
  echo "✓ 视频资源就绪：$DEST/bin"
  exit 0
fi

echo "视频资源缺失或不可执行，重新收集到 $DEST ..."
bash "$PROJECT_DIR/scripts/collect_macos_tools.sh" "$(dirname "$DEST")"

run_ok "$DEST/bin/ffmpeg" || { echo "✗ 收集后 ffmpeg 仍不可执行" >&2; exit 1; }
run_ok "$DEST/bin/ffprobe" || { echo "✗ 收集后 ffprobe 仍不可执行" >&2; exit 1; }
echo "✓ 视频资源收集完成：$DEST/bin"
