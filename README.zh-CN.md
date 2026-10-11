<div align="center">

**[English](README.md) · 简体中文**

# Sizqo

**免费开源、本地运行的图片 / 视频压缩工具 —— 拖进去，压缩好，还给你。**

[![Release](https://img.shields.io/github/v/release/misswell/Sizqo)](https://github.com/misswell/Sizqo/releases/latest)
[![Platform](https://img.shields.io/badge/platform-macOS-blue)](https://github.com/misswell/Sizqo/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-orange)](https://tauri.app)

![Sizqo](assets/banner.png)

**[下载最新版](https://github.com/misswell/Sizqo/releases/latest)** · **[问题反馈](https://github.com/misswell/Sizqo/issues)**

</div>

---

Sizqo 把常用的媒体压缩能力装进一个 macOS 桌面应用：**图片和视频都支持，所有压缩都在你的电脑上完成，素材永不上传**，断网也能用。适合日常整理截图、设计稿、公众号配图、网站图片、录屏和相机素材。

- **素材不上云** —— 本机处理，不注册、不登录、没有上传这一步
- **开箱即用** —— 压缩引擎随应用内置，无需安装任何命令行工具
- **批量拖拽** —— 图片、视频或文件夹整批拖入，自动递归子目录，按文件内容识别媒体类型
- **原图保护** —— 覆盖前先备份真原文件，任何时候都能一键恢复
- **免费开源** —— MIT 协议，无内购、无激活

## 界面一览

| 亮色 | 暗色 |
|------------|-----------|
| ![亮色](assets/screenshot-light.png) | ![暗色](assets/screenshot-dark.png) |

独立对比窗口，拖动分割线逐像素对比：

![对比窗口](assets/compare.png)

## 核心能力

### 图片

- **两种模式**：「高级压缩」自定义质量、力度和输出格式（原格式或转 JPEG/PNG/WebP/AVIF）；macOS 上还可用「系统转换」，对齐 Finder 的转换流程。
- **智能模式**：按每张图自动挑算法和参数，也可以手动指定引擎和力度。
- **可视化对比**：独立窗口拖动分割线，压缩前后逐像素对比。

### 视频（Direct 版）

- **预设**：快速（硬件 H.264/HEVC）、均衡、高质量（x264/x265 CRF）、目标体积（两遍编码）、兼容优先，以及完全自定义的编码参数。
- **编码器**：支持时优先硬件 VideoToolbox（H.264/HEVC），软件 x264、x265、SVT-AV1（AV1）可选；编码器回退会明确展示。
- **输出容器**：MP4、MOV、MKV、WebM。音轨、字幕、章节、附件要么保留，要么明确告知被移除，绝不悄悄丢。
- **诚实的结果**：只有当输出有效、可播放且确实更小时才算“压缩成功”；HDR / 杜比视界、10bit、Alpha、隔行扫描等输入会被拒绝并解释原因，不做静默色调映射。
- **逐任务进度**：编码速度、预计剩余时间，以及有上限的诊断日志。
- **同帧对比**：选一个时间点，源视频和输出视频的帧并排对比。

### 原文件保护

- **覆盖前备份**：替换前先把原文件备份到本地；备份存不下就不替换。
- **一键恢复**：从结果行或历史记录恢复。压缩后如果文件又被其他应用改过，恢复前会先询问。
- **压缩历史**：记录跨启动保留，并带有媒体类型。每条记录只提供当前真实可用的操作：另存为、对比、恢复原图（或删除压缩结果）、在访达中显示、复制日志。
- **备份保留期**：默认“不保留”（退出时清理记录与备份），也可以选 1/3/7/14/30 天。清理只碰 Sizqo 自己的历史和备份，绝不碰你的媒体文件。
- **崩溃安全**：替换操作有预提交日志。意外退出后，下次启动自动完成或回滚被中断的操作；仍可能是唯一原件的备份绝不会被清掉。

### 队列与任务

- **暂停 / 继续 / 停止**：暂停不再启动新文件，正在压缩的跑完；停止结束这一轮，剩余文件留在队列里；继续从原进度接着跑。失败文件可逐个重试。
- **取消当前视频**：正在编码的视频可以单独取消，不影响队列其他任务。
- **CPU 上限**：自动调度，或限制并行份数（同时文件数 × 编码线程）给其他应用让路。视频同时只编一个，解码/编码线程数同样计入这份预算。
- **自动压缩**：导入后自动开始压缩，可在设置里开关。

### 界面

- 🌗 亮色 / 暗色 / 跟随系统外观；压缩设置默认折叠。
- 🌐 简体中文和英语，默认跟随系统语言，可在设置里固定。

## 下载与安装

| 渠道 | 适合 | 获取方式 |
|---------|----------|---------------|
| [GitHub Releases](https://github.com/misswell/Sizqo/releases/latest) | 完整视频压缩与最新功能 | 下载对应架构的 macOS 安装包 |

GitHub Releases 提供 Apple Silicon、Intel 与 Universal 版 DMG。直发版已做 Developer ID 签名与公证。

### 快速上手

1. 打开 Sizqo，把图片、视频或文件夹拖进窗口，或点「选择媒体文件」/「选择文件夹」。
2. 展开「压缩设置」（图片）或「视频设置」调整编码器、质量、容器和输出位置；不改就用默认值。
3. 点「开始压缩」。过程中随时可以暂停、停止或继续追加文件。
4. 点结果行的「对比查看」检查前后效果。视频行还提供播放原/输出视频、同帧对比、查看日志与恢复。
5. 「覆盖原文件」模式下随时可恢复原图；「历史」页保留更早的记录。

### macOS 提示“已损坏，无法打开”

直发版已经签名、公证并装订票据，正常安装不会遇到这个提示。如果系统仍然拦截，可以在终端移除隔离属性：

```bash
sudo xattr -dr com.apple.quarantine /Applications/Sizqo.app
```

## 和在线工具相比

| 能力 | Sizqo | 在线压缩 | 命令行工具 |
|------------|:----------:|:------------------:|:------------------:|
| 文件留在本机 | ✅ | ❌ | ✅ |
| 图片和视频一个队列 | ✅ | 少见 | 需要多套工具 |
| 文件夹递归批量 | ✅ | 通常受限 | 需要写脚本 |
| 开箱即用 | ✅ | ✅ | 通常要装依赖 |
| 可视化前后对比 | ✅ | 有时有 | ❌ |
| 原文件备份与一键恢复 | ✅ | ❌ | ❌ |
| 断网可用 | ✅ | ❌ | ✅ |
| 免费开源 | ✅ | 部分免费 | 大多免费 |

## 技术架构

Sizqo 使用 **Tauri 2**（Rust 后端 + 原生 HTML/CSS/JavaScript 前端，前端无构建步骤），另有一个**原生 Swift** 版本。最低系统版本：Direct 11.0、App Store 12.0、Swift 原生 13.0。

### 三条发布线

| 线 | Bundle ID | 压缩后端 | 视频状态 |
|-------|-----------|---------------------|--------------|
| Direct | `com.misswell.sizqo` | 内置 FFmpeg + 图片 CLI 工具（`cli-backends`，默认） | 完整 FFmpeg 管线 |
| App Store | `com.misswell.sizqo.appstore` | 进程内 Rust 库（`inproc-backends`），沙盒 | 原生桥接与沙盒测试通过前，运行时明确禁用 |
| Swift 原生 | `com.misswell.sizqo.swift` | ImageIO / 内置图片编码器 | 目前仅图片；视频尚未实现 |

每条线的实际验证状态记录在 [docs/validation.md](docs/validation.md) —— 构建跑不了的功能要么隐藏、要么说明原因，绝不造假选项。

- App Store 版不打包、不启动第三方可执行文件：图片格式全部走进程内 Rust 库，用户文件通过安全作用域书签访问。
- 图片引擎能力（PNG、JPEG、WebP、AVIF、GIF）三条线一致；App Store 版对图片使用进程内 crate。
- 三条线各自独立的记录、备份与设置根目录；原文件备份绝不放在临时目录。
- 工程规则见 [AGENTS.md](AGENTS.md)，视频任务协议见 [core/media-task-protocol.md](core/media-task-protocol.md)。

### 压缩引擎一览

| 媒体 | Direct (CLI) | App Store (进程内 crate) | Swift 原生 |
|--------|--------------|-------------------------------|--------------------|
| PNG | pngquant / oxipng | imagequant / oxipng | 与 Direct 共用二进制 |
| JPEG | cjpeg (mozjpeg) | mozjpeg | cjpeg (mozjpeg) |
| WebP | cwebp | webp | cwebp |
| AVIF | avifenc | ravif | avifenc |
| GIF | gifsicle（减色） | image crate 重编码 | gifsicle（减色） |
| 视频 | 内置 FFmpeg（x264 / x265 / SVT-AV1 / VideoToolbox） | 原生桥接验证通过前禁用 | 尚未实现 |

Direct 需要的 CLI 工具与库都打包在应用 `Contents/Resources` 里，用户无需自己安装。内置 FFmpeg 配置不含 GPL 组件；第三方许可细节汇总在 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## 本地开发

### 环境要求

- [Rust](https://rustup.rs/) 1.77+ 与 [Tauri CLI](https://tauri.app/) 2.x（`cargo install tauri-cli --version "^2.0"`）
- 开发机需要 FFmpeg（含 ffprobe）与图片 CLI 工具；正式包会为用户内置：

```bash
brew install ffmpeg pngquant oxipng mozjpeg gifsicle webp jpeg-xl libavif
```

### 常用命令

```bash
cargo tauri dev                 # 开发模式（默认 Direct 特性）

bash scripts/build_all.sh       # 构建 Direct + App Store 版（未签名）
SIGN=1 bash scripts/build_all.sh

cargo tauri build               # 构建 Direct 正式版
bash scripts/notarize.sh        # Direct：构建、签名、公证、装订、打包 DMG
bash scripts/build_appstore.sh  # App Store：构建、签名、打包 PKG

# 测试（发布前本地全部跑一遍）
cargo test                                                    # 默认 Rust 特性
cargo test --features inproc-backends                         # 与 App Store 共用的图片引擎
cargo test --no-default-features --features inproc-backends   # App Store 特性集
npm run test:frontend                                         # 前端队列、媒体、历史与设置检查
bash scripts/test_swift_history.sh                            # Swift 历史、备份与暂停检查
```

### 项目结构

```
Sizqo/
├── frontend/                       # 前端（纯 HTML/CSS/JS，无构建步骤）
│   ├── index.html / app.js          # 主窗口（媒体工作台）
│   ├── video-settings.js            # 视频设置面板
│   └── compare.html / compare_window.js  # 独立对比窗口
├── src-tauri/                       # 两条 Tauri 线共用的 Rust 后端
│   ├── src/
│   │   ├── engine.rs               # 图片压缩引擎（CLI 后端）
│   │   ├── engine_inproc.rs        # 进程内图片引擎（不启动子进程）
│   │   ├── video/                  # 视频探测、能力矩阵、计划、编码、校验、事务
│   │   ├── video_commands.rs       # 媒体 Tauri 命令（探测、会话、进度、帧）
│   │   ├── commands.rs             # 图片 Tauri 命令与 CompressionScheduler
│   │   ├── history.rs              # 历史、备份与恢复服务
│   │   └── output_transaction.rs   # 替换日志与崩溃恢复
│   └── tauri.conf*.json             # Direct / App Store 配置
├── swift/                           # 原生 Swift 版本，独立的存储根目录
├── core/                            # 状态机与媒体任务协议文档
├── tests/                           # 前端检查（Node.js 直接运行）
├── scripts/                         # 构建、签名、公证与验证脚本
└── AGENTS.md                        # 工程规则与发布线要求
```

## License

[MIT](LICENSE)。内置的第三方组件（FFmpeg、图片编码器）各自遵循其许可 —— 见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

---

## 👨‍💻 作者的其他开源项目

**[MacPilot](https://github.com/misswell/MacPilot)** —— 零第三方依赖的原生 macOS 菜单栏工具箱，集应用自动退出、BLE 靠近解锁、窗口切换、剪贴板历史、平滑滚动、画中画、录屏、截屏钉图等于一身。[免费下载](https://github.com/misswell/MacPilot/releases/latest)，欢迎点个 ⭐
