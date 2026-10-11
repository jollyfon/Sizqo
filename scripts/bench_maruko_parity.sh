#!/bin/bash
# Sizqo vs 小丸工具箱(粉丝致敬版) 压缩效果对比基准。
# 两侧用同一个 ffmpeg 二进制(本仓库收集的 9.0.2),唯一变量是命令形态:
# argv 按 MarukoToolbox-Rewrite 的 build_compress_command(ffmpeg_utils.py)
# 与本仓库 src-tauri/src/video/plan.rs 的 EncodePlan::argv 逐参数复刻。

set -uo pipefail
FF="/Users/guofeng/Code/solo/Sizqo/src-tauri/resources/video/bin/ffmpeg"
OUT=/tmp/maruko-bench
rm -rf "$OUT"; mkdir -p "$OUT"; cd "$OUT"
N=8   # Sizqo 编码线程(=自动预算)

echo "== 生成测试素材(1280x720/30fps/15s/yuv420p + AAC 128k) =="
"$FF" -v error -f lavfi -i "gradients=s=1280x720:c0=0x224488:c1=0xEE8844:x0=100:y0=100:x1=1100:y1=600:nb_colors=3:speed=0.05" \
  -f lavfi -i "sine=frequency=660:duration=15" \
  -t 15 -r 30 -pix_fmt yuv420p -c:v libx264 -preset veryfast -crf 18 -c:a aac -b:a 128k -shortest src_anime.mp4 || exit 1
"$FF" -v error -f lavfi -i "testsrc2=s=1280x720:r=30:d=15" -f lavfi -i "sine=frequency=440:duration=15" \
  -pix_fmt yuv420p -c:v libx264 -preset veryfast -crf 18 -c:a aac -b:a 128k -shortest src_live.mp4 || exit 1
"$FF" -v error -f lavfi -i "mandelbrot=s=1280x720:r=30" -f lavfi -i "sine=frequency=520:duration=15" \
  -t 15 -pix_fmt yuv420p -c:v libx264 -preset veryfast -crf 18 -c:a aac -b:a 128k -shortest src_screen.mp4 || exit 1
"$FF" -v error -f lavfi -i "testsrc2=s=1920x1080:r=30:d=15" -f lavfi -i "sine=frequency=440:duration=15" \
  -pix_fmt yuv420p -c:v libx264 -preset veryfast -crf 18 -c:a aac -b:a 128k -shortest src_1080.mp4 || exit 1

bench_pair() { # id src enc crf scale_mode
  local id="$1" src="$2" enc="$3" crf="$4" scale_mode="$5"
  local m_vf=() s_vf=(-vf "scale=1280:720")
  if [ "$scale_mode" = "down" ]; then m_vf=(-vf "scale=1280:-2"); fi
  "$FF" -hide_banner -n -i "$src" ${m_vf[@]+"${m_vf[@]}"} -c:v "$enc" -preset medium -crf "$crf" -c:a copy -map_metadata 0 "M_$id.mp4" >"m_$id.log" 2>&1
  echo "m_$id rc=$?"
  local extra=()
  if [ "$enc" = "libx265" ]; then extra=( -x265-params "pools=$N:frame-threads=1" ); fi
  "$FF" -hide_banner -nostdin -n -protocol_whitelist file,pipe -threads $N -i "$src" -map 0:v:0 -c:v "$enc" -threads $N -filter_threads $N \
    -pix_fmt yuv420p ${s_vf[@]+"${s_vf[@]}"} -metadata:s:v:0 rotate=0 -preset medium ${extra[@]+"${extra[@]}"} -crf "$crf" -fps_mode passthrough \
    -map '0:a?' -c:a copy -map_metadata 0 -map_chapters 0 -movflags +faststart -f mp4 "S_$id.mp4" >"s_$id.log" 2>&1
  echo "s_$id rc=$?"
}

for src_name in anime live screen; do
  bench_pair "${src_name}_x264" "src_$src_name.mp4" libx264 23 keep
done
for src_name in anime live screen; do
  bench_pair "${src_name}_x265" "src_$src_name.mp4" libx265 28 keep
done
bench_pair "1080p_x264" "src_1080.mp4" libx264 23 down

echo "== SVT-AV1 CRF30(maruko 固定 preset6;sizqo medium→8 / slow→4)=="
"$FF" -hide_banner -n -i src_anime.mp4 -c:v libsvtav1 -preset 6 -crf 30 -c:a copy -map_metadata 0 M_anime_av1.mkv >m_anime_av1.log 2>&1; echo "m_av1 rc=$?"
"$FF" -hide_banner -nostdin -n -protocol_whitelist file,pipe -threads $N -i src_anime.mp4 -map 0:v:0 -c:v libsvtav1 -threads $N -filter_threads $N \
  -pix_fmt yuv420p -vf "scale=1280:720" -metadata:s:v:0 rotate=0 -preset 8 -svtav1-params "lp=$N" -crf 30 -fps_mode passthrough \
  -map '0:a?' -c:a copy -map_metadata 0 -map_chapters 0 -f matroska S_anime_av1_p8.mkv >s_anime_av1_p8.log 2>&1; echo "s_av1_p8 rc=$?"
"$FF" -hide_banner -nostdin -n -protocol_whitelist file,pipe -threads $N -i src_anime.mp4 -map 0:v:0 -c:v libsvtav1 -threads $N -filter_threads $N \
  -pix_fmt yuv420p -vf "scale=1280:720" -metadata:s:v:0 rotate=0 -preset 4 -svtav1-params "lp=$N" -crf 30 -fps_mode passthrough \
  -map '0:a?' -c:a copy -map_metadata 0 -map_chapters 0 -f matroska S_anime_av1_p4.mkv >s_anime_av1_p4.log 2>&1; echo "s_av1_p4 rc=$?"

echo "== 两遍 2000k(maruko: base+-pass;sizqo: twoPass argv)=="
"$FF" -hide_banner -n -i src_live.mp4 -c:v libx264 -preset medium -crf 23 -b:v 2000k -c:a copy -map_metadata 0 -pass 1 -an -f null /dev/null >m_2p1.log 2>&1; echo "m2p1 rc=$?"
"$FF" -hide_banner -n -i src_live.mp4 -c:v libx264 -preset medium -crf 23 -b:v 2000k -c:a copy -map_metadata 0 -pass 2 M_live_2pass.mp4 >m_2p2.log 2>&1; echo "m2p2 rc=$?"
"$FF" -hide_banner -nostdin -n -protocol_whitelist file,pipe -threads $N -i src_live.mp4 -map 0:v:0 -c:v libx264 -threads $N -filter_threads $N \
  -pix_fmt yuv420p -vf "scale=1280:720" -metadata:s:v:0 rotate=0 -preset medium -b:v 2000k -fps_mode passthrough \
  -pass 1 -passlogfile sizqo_stats -an -sn -f null - >s_2p1.log 2>&1; echo "s2p1 rc=$?"
"$FF" -hide_banner -nostdin -n -protocol_whitelist file,pipe -threads $N -i src_live.mp4 -map 0:v:0 -c:v libx264 -threads $N -filter_threads $N \
  -pix_fmt yuv420p -vf "scale=1280:720" -metadata:s:v:0 rotate=0 -preset medium -b:v 2000k -fps_mode passthrough \
  -pass 2 -passlogfile sizqo_stats -map '0:a?' -c:a copy -map_metadata 0 -map_chapters 0 -movflags +faststart -f mp4 S_live_2pass.mp4 >s_2p2.log 2>&1; echo "s2p2 rc=$?"

echo; echo "== 汇总(delta = (Sizqo-小丸)/小丸)=="
printf "%-24s %10s %10s %9s\n" "case" "maruko" "sizqo" "delta"
for id in anime_x264 live_x264 screen_x264 anime_x265 live_x265 screen_x265 1080p_x264; do
  M=$(stat -f%z "M_$id.mp4" 2>/dev/null || echo 0)
  S=$(stat -f%z "S_$id.mp4" 2>/dev/null || echo 0)
  D=$(python3 -c "print(f'{($S-$M)/$M*100:+.2f}%' if $M else 'n/a')")
  printf "%-24s %10d %10d %9s\n" "$id" "$M" "$S" "$D"
done
M=$(stat -f%z M_anime_av1.mkv 2>/dev/null || echo 0)
S8=$(stat -f%z S_anime_av1_p8.mkv 2>/dev/null || echo 0)
S4=$(stat -f%z S_anime_av1_p4.mkv 2>/dev/null || echo 0)
python3 - <<PYEOF
m,s8,s4=$M,$S8,$S4
print(f"{'anime_av1(p6 vs p8)':<24} {m:>10} {s8:>10} {(s8-m)/m*100:+9.2f}%")
print(f"{'anime_av1(p6 vs p4)':<24} {m:>10} {s4:>10} {(s4-m)/m*100:+9.2f}%")
PYEOF
M=$(stat -f%z M_live_2pass.mp4 2>/dev/null || echo 0)
S=$(stat -f%z S_live_2pass.mp4 2>/dev/null || echo 0)
python3 - <<PYEOF
m,s=$M,$S
print(f"{'live_2pass_2000k':<24} {m:>10} {s:>10} {(s-m)/m*100:+9.2f}%")
PYEOF

echo; echo "== SSIM(输出 vs 源)=="
for id in anime_x264 live_x264 screen_x264 anime_x265 live_x265 screen_x265; do
  SN="${id%%_*}"
  for side in M S; do
    F="${side}_$id.mp4"
    [ -f "$F" ] || continue
    SS=$("$FF" -hide_banner -i "$F" -i "src_$SN.mp4" -lavfi "[0:v][1:v]ssim" -f null - 2>&1 | grep Parsed_ssim | tail -1 | sed -E 's/.*All:([0-9.]+).*/\1/')
    printf "%-26s %s\n" "${side}_$id" "$SS"
  done
done
echo "BENCH DONE"
