# Validation record

This file records the observed build/test/runtime status of the three Sizqo
tracks. Rules: a feature is "supported" only with a command and its real
outcome recorded here; pending or blocked work is never reported as passed.
Update this file whenever a track's capability changes.

Last updated: 2026-10-11

## Automated checks

| Check | Command | Result |
|-------|---------|--------|
| Rust tests, Direct features | `cargo test` (in `src-tauri`) | ✅ 85 unit + 6 integration passed |
| Rust tests, shared in-process engines | `cargo test --features inproc-backends` | ✅ 86 unit + 6 integration passed |
| Rust tests, App Store feature set | `cargo test --no-default-features --features inproc-backends` | ✅ 86 unit + 6 integration passed |
| Frontend suites (queue, sessions, media import, video contract, i18n, history, CPU limit, settings, compare, update panel, parity) | `npm run test:frontend` (15 suites, incl. `tests/video-contract.cjs`, `tests/app-slices.cjs`) | ✅ all pass |
| Swift history / backup / pause / queue / settings self-checks | `bash scripts/test_swift_history.sh` | ✅ all pass |

## Track: Direct (`com.misswell.sizqo`)

- Image pipeline: inherited OctoShrink engine, covered by the Rust suites and
  frontend suites above. No regression observed.
- Video pipeline (`src-tauri/src/video/`, `video_commands.rs`):
  - Probe / capability trials / EncodePlan / streaming encode / progress /
    cancellation / validation / transaction are unit-tested (87 unit tests).
  - Software encoder set: x264, x265, SVT-AV1, VP9; containers MP4, MOV,
    MKV, WebM, TS; audio policies copy/AAC/Opus/MP3/FLAC/none. Real runtime
    validation (2026-10-11): VP9+WebM+Opus, x264+TS+MP3, and x264+MKV+FLAC
    encoded with the collected binary, probed correct (format/codec), and
    passed full-decode checks.
  - Real encode end-to-end with the collected runtime (2026-10-11, arm64):
    generated a 1280x720/30fps H.264+AAC 5s MP4 (4.7 MB), re-encoded it with
    the plan-shaped argv (libx264 CRF 28, AAC 160k, `+faststart`) through the
    collected bundled binary; output 937 KB (80% smaller), `ffprobe` reports
    exactly 5.000000 s, and a full `-xerror` decode of the output passed.
  - Full acceptance matrix (AGENTS.md §11, items 1–12 incl. HDR/VFR/multi-track
    fixtures, kill-during-commit recovery, 20 GB sources): **pending** —
    needs real user-accepted fixtures and signed-runtime runs.
- Bundled runtime: `scripts/collect_macos_tools.sh` collects `ffmpeg` 9.0.2 +
  `ffprobe` (Homebrew build, GPL configuration: `--enable-gpl --enable-libx264
  --enable-libx265 --enable-libsvtav1`; encoders verified present in the
  collected copy) into `src-tauri/resources/video/{bin,lib}` (19 dylibs),
  ad-hoc signed so they execute; `scripts/notarize.sh`, `scripts/package.sh`
  and `scripts/video_collect_ffmpeg.sh` stage/verify them. The GPL
  configuration is **not publishable** until the review in
  [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) is done.
- Unsigned build (2026-10-11): `bash scripts/build_all.sh` produced
  `src-tauri/target/release/bundle/macos/Sizqo_direct.app` (69 MB); the
  bundled `ffmpeg`/`ffprobe`/`pngquant` were executed from inside the built
  `.app` successfully. Signed-runtime check and GUI launch check: **pending**.

## Track: App Store (`com.misswell.sizqo.appstore`)

- Image pipeline: in-process engines build and test under
  `--no-default-features --features inproc-backends`.
- Video: intentionally unavailable at runtime; the capability matrix reports
  the reason instead of showing fake controls
  (`video::capabilities::NATIVE_UNAVAILABLE`). This is the documented,
  accurate runtime-disabled state; no native bridge exists yet.
- Unsigned build (2026-10-11): `bash scripts/build_all.sh` produced
  `src-tauri/target/release/bundle/macos/Sizqo.app` (17 MB); verified the
  bundle contains **zero** third-party binaries/dylibs (`Resources/bin`,
  `Resources/lib`, `Resources/video` all absent).
- Signed sandbox runtime check: **pending** (release engineering).

## Track: Swift native (`com.misswell.sizqo.swift`)

- Module renamed `OctoShrinkSwift` → `SizqoSwift`; independent Application
  Support root with `sizqo.`-prefixed settings keys.
- Image functionality: inherited, verified by `scripts/test_swift_history.sh`.
- Build (2026-10-11): `bash scripts/build_swift.sh` produced
  `swift/.build/Sizqo_swift.app` (22 MB, 6 CLI tools + 28 dylibs).
- Video: **not implemented**. No VideoProbe/VideoEncoder exists in
  `swift/Sources/SizqoSwift`; do not advertise video for this track.

## Release gate (per [AGENTS.md](../AGENTS.md))

Before any tag/Release: full three-track build, signed Direct runtime check,
signed sandbox check, real fixture video matrix (AGENTS.md §11 items 1–12),
FFmpeg licensing review. A local commit is not a GitHub push; every published
tag must link its artifacts here with the exact commands used.
