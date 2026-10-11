<div align="center">

# Sizqo — Local Image & Video Compressor

**Free, open-source desktop media compression — drop files in, get smaller files back.**

[English](README.md) · [简体中文](README.zh-CN.md)

[![Release](https://img.shields.io/github/v/release/misswell/Sizqo)](https://github.com/misswell/Sizqo/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS-blue)](https://github.com/misswell/Sizqo/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-orange)](https://tauri.app)

![Sizqo](assets/banner.png)

**[Latest release](https://github.com/misswell/Sizqo/releases/latest)** · **[Report an issue](https://github.com/misswell/Sizqo/issues)**

</div>

---

Sizqo is a free, open-source compressor for images **and videos** on macOS. All processing happens on your computer: your media is never uploaded, and the app works offline.

## Why Sizqo

Online compressors upload your files to someone else's server. Command-line tools require dependencies and memorized options. Sizqo puts everyday media compression in a desktop app:

- **Your files stay private** — processing is local; no account, sign-in, or upload
- **Ready to use** — compression engines are bundled; no command-line tools to install
- **Batch drag and drop** — add images, videos, or folders; nested folders expand automatically and media types are detected from file contents
- **Protects originals** — saves a real backup before replacing and can restore it later
- **Free and open source** — MIT licensed, with no purchase or activation

## Screenshots

| Light mode | Dark mode |
|------------|-----------|
| ![Light mode](assets/screenshot-light.png) | ![Dark mode](assets/screenshot-dark.png) |

Compare the source and compressed image pixel by pixel in a separate window by dragging the split line:

![Comparison window](assets/compare.png)

## Features

### Images

- **Two modes:** Advanced Compression lets you set quality, effort, and output format (keep the source format or convert to JPEG, PNG, WebP, or AVIF). On macOS, System Conversion follows Finder's image conversion workflow, with JPEG, PNG, and HEIF output, size presets, and metadata retention.
- **Smart compression:** chooses an algorithm and settings for each image. You can also select an engine and effort manually.
- **Visual comparison:** drag the split line to compare the source and compressed image side by side.

### Videos (Direct edition)

- **Presets:** Fast (hardware H.264/HEVC), Balanced, High quality (x264/x265 CRF), Target size (two-pass), Compatibility-first, and fully custom encoder settings.
- **Encoders:** hardware H.264/HEVC through VideoToolbox when available, plus software x264, x265, SVT-AV1 (AV1), and VP9, with automatic fallback shown explicitly.
- **Output containers:** MP4, MOV, MKV, WebM, and TS. Audio can be copied or converted to AAC, Opus, MP3, or FLAC (lossless, MKV). Tracks, subtitles, chapters, and attachments are preserved or explicitly dropped — never silently lost.
- **Honest results:** an output is "compressed" only when it is valid, playable, and smaller than the source. HDR / Dolby Vision, 10-bit, alpha, and interlaced inputs are refused with an explanation instead of silently tone-mapped.
- **Per-task progress:** encoding speed, estimated remaining time, and a bounded diagnostics log for every task.
- **Frame comparison:** pick a timestamp and compare the source and output frames side by side.

### Original protection

- **Backup before replacing:** Sizqo saves the original file locally before replacing it. If the backup cannot be saved, the replacement is skipped.
- **Restore in one click:** restore from a result or history entry. If another app changed the file after compression, Sizqo asks before overwriting it.
- **Compression history:** records persist across launches and carry their media type. Each entry offers the actions that are currently available: save a copy, compare, restore the original (or remove the compressed result), reveal in Finder, and copy the log.
- **Backup retention:** the default is “Don't keep” (history and backups are cleaned up when the app quits); you can also keep them for 1, 3, 7, 14, or 30 days. Cleanup only touches Sizqo's own history and backups, never your files.
- **Crash safety:** replacement operations are journaled. After an unexpected exit, the next launch completes or rolls back the interrupted operation; a backup is never swept while it may be the only copy of your original.

### Queue and jobs

- **Pause, resume, and stop:** pause waits for files that have not started while current work finishes. Stop ends the current pass and leaves the remaining files queued; Resume continues from there without losing progress. Failed files can be retried individually.
- **Cancel current video:** an in-flight encode can be cancelled on its own without touching the rest of the queue.
- **CPU limit:** choose automatic scheduling or limit parallel work (files at once × encoder threads) to leave capacity for other apps. Video encodes run one at a time with their decoder/encoder threads clamped to the same budget.
- **Automatic processing:** optionally start compression as soon as files are imported.

### Interface

- 🌗 Light, dark, or system appearance; compression settings are collapsed by default.
- 🌐 English and Simplified Chinese. The interface follows the system language by default, and you can choose a language in Settings.

## Download and install

| Channel | Best for | How to get it |
|---------|----------|---------------|
| [GitHub Releases](https://github.com/misswell/Sizqo/releases/latest) | Full video compression and the latest features | Download the macOS build for your architecture |

GitHub Releases provides macOS Apple Silicon and Intel builds (plus a Universal DMG). The direct-download macOS app is Developer ID signed and notarized.

### Quick start

1. Open Sizqo and drop images, videos, or folders into the window, or choose **Choose Media** / **Choose Folder**.
2. Expand **Compression settings** (images) or **Video settings** to adjust encoder, quality, container, and output location. Otherwise, start with the defaults.
3. Select **Start compression**. You can pause, stop, or add files at any time.
4. Select **Compare** on a result row to inspect source and output. Video rows offer playback of both files, frame comparison, the diagnostics log, and restore.
5. In Replace mode, restore the original at any time. The **History** page also provides earlier records.

### macOS says “damaged and can't be opened”

The direct-download edition is signed, notarized, and stapled, so a normal install should open without this warning. If your system still complains, you can remove the quarantine attribute in Terminal:

```bash
sudo xattr -dr com.apple.quarantine /Applications/Sizqo.app
```

## Compared with online and command-line tools

| Capability | Sizqo | Online compressors | Command-line tools |
|------------|:----------:|:------------------:|:------------------:|
| Files stay on your device | ✅ | ❌ | ✅ |
| Images and videos in one queue | ✅ | Rarely | Requires multiple tools |
| Recursive folder batches | ✅ | Usually limited | Requires a script |
| Ready to use | ✅ | ✅ | Usually requires installation |
| Visual before-and-after comparison | ✅ | Sometimes | ❌ |
| Original backup and one-click restore | ✅ | ❌ | ❌ |
| Works offline | ✅ | ❌ | ✅ |
| Free and open source | ✅ | Some are free | Most are free |

## Technical architecture

Sizqo uses **Tauri 2** (a Rust backend with a native HTML/CSS/JavaScript frontend and no frontend build step) and also ships a **native Swift** edition. Minimum macOS versions: 11.0 for Direct, 12.0 for App Store, and 13.0 for Swift Native.

### Distribution tracks

| Track | Bundle ID | Compression backend | Video status |
|-------|-----------|---------------------|--------------|
| Direct | `com.misswell.sizqo` | Bundled FFmpeg + image CLI tools (`cli-backends`, the default) | Full FFmpeg pipeline |
| App Store | `com.misswell.sizqo.appstore` | In-process Rust libraries (`inproc-backends`), sandboxed | Disabled at runtime until the native bridge and sandbox tests pass |
| Swift Native | `com.misswell.sizqo.swift` | ImageIO / bundled image encoders | Images only today; video is not implemented yet |

The exact, observed status of every track is recorded in [docs/validation.md](docs/validation.md) — options that a build cannot run are hidden or explained at runtime, never faked.

- The App Store edition does not package or launch third-party executables. It uses in-process Rust libraries for all image formats and accesses user files through security-scoped bookmarks.
- Image engine capabilities (PNG, JPEG, WebP, AVIF, GIF) are shared across the tracks; the App Store edition uses in-process crates for them.
- Each track has its own history, backup, and settings root. Original backups are never stored in temporary folders.
- Engineering rules are documented in [AGENTS.md](AGENTS.md); the video task protocol lives in [core/media-task-protocol.md](core/media-task-protocol.md).

### Compression engine overview

| Media | Direct (CLI) | App Store (in-process crates) | Swift Native |
|--------|--------------|-------------------------------|--------------------|
| PNG | pngquant / oxipng | imagequant / oxipng | Shared binaries with Direct |
| JPEG | cjpeg (mozjpeg) | mozjpeg | cjpeg (mozjpeg) |
| WebP | cwebp | webp | cwebp |
| AVIF | avifenc | ravif | avifenc |
| GIF | gifsicle (color reduction) | image crate re-encoding | gifsicle (color reduction) |
| Video | Bundled FFmpeg (x264 / x265 / SVT-AV1 / VP9 / VideoToolbox) | Disabled until native bridge is verified | Not yet implemented |

The CLI tools and libraries required by Direct are bundled in the app's `Contents/Resources` — users do not need to install anything. FFmpeg is built without GPL components for this bundled configuration; licensing details are collected in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Development

### Requirements

- [Rust](https://rustup.rs/) 1.77+ and [Tauri CLI](https://tauri.app/) 2.x (`cargo install tauri-cli --version "^2.0"`)
- FFmpeg (with ffprobe) and the image CLI tools for development; release apps bundle them for users:

```bash
brew install ffmpeg pngquant oxipng mozjpeg gifsicle webp jpeg-xl libavif
```

### Common commands

```bash
cargo tauri dev                 # Development mode (Direct feature by default)

bash scripts/build_all.sh       # Build Direct + App Store editions (unsigned)
SIGN=1 bash scripts/build_all.sh

cargo tauri build               # Build the Direct release edition
bash scripts/notarize.sh        # Direct: build, sign, notarize, staple, and package a DMG
bash scripts/build_appstore.sh  # App Store: build, sign, and package a PKG

# Tests (run locally before publishing)
cargo test                                                    # Default Rust feature
cargo test --features inproc-backends                         # Image engines shared with the App Store build
cargo test --no-default-features --features inproc-backends   # App Store feature set
npm run test:frontend                                         # Frontend queue, media, history, and settings checks
bash scripts/test_swift_history.sh                            # Swift history, backup, and pause checks
```

### Project structure

```
Sizqo/
├── frontend/                       # Frontend (plain HTML/CSS/JS; no build step)
│   ├── index.html / app.js          # Main window (media workbench)
│   ├── video-settings.js            # Video settings panel
│   └── compare.html / compare_window.js  # Separate comparison window
├── src-tauri/                       # Rust backend shared by both Tauri tracks
│   ├── src/
│   │   ├── engine.rs               # Image compression engine (CLI backend)
│   │   ├── engine_inproc.rs        # In-process image engine (no spawned processes)
│   │   ├── video/                  # Video probe, capabilities, plan, encode, validation, transaction
│   │   ├── video_commands.rs       # Media Tauri commands (probe, sessions, progress, frames)
│   │   ├── commands.rs             # Image Tauri commands and CompressionScheduler
│   │   ├── history.rs              # History, backups, and restore service
│   │   └── output_transaction.rs   # Replacement journal and crash recovery
│   └── tauri.conf*.json             # Direct / App Store configurations
├── swift/                           # Native Swift edition with its own storage root
├── core/                            # State machine and media task protocol documents
├── tests/                           # Frontend checks (run directly with Node.js)
├── scripts/                         # Build, signing, notarization, and verification
└── AGENTS.md                        # Engineering rules and distribution-track requirements
```

## License

[MIT](LICENSE). Bundled third-party components (FFmpeg, image encoders) are covered by their own licenses — see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

---

## Other open-source projects by the author

**[MacPilot](https://github.com/misswell/MacPilot)** — an open-source native macOS menu-bar toolkit with zero third-party dependencies. It combines app auto-quit rules, BLE proximity unlock, window switching, clipboard history, smooth scrolling, picture-in-picture, screen recording, screenshot pinning, and more. [Download for free](https://github.com/misswell/MacPilot/releases/latest); stars are always welcome ⭐
