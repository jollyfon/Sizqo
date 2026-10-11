# Sizqo 推广包

这份文档用于发布、投稿和社交媒体推广。所有文案都基于当前仓库功能：本地图片 + 视频压缩、批量处理、Tauri 2 + Rust、内置压缩引擎（图片 7 件套 + FFmpeg）、支持压缩前后对比和原文件恢复。

## 一句话介绍

Sizqo 是一个免费开源、本地运行的图片 / 视频压缩工具。拖入图片、视频或文件夹，它会在电脑上批量压缩，素材永不上传，并支持压缩前后对比。

## 短介绍

Sizqo 是一个免费开源的桌面媒体压缩工具：图片支持 PNG、JPG、GIF、WebP、BMP，可输出 WebP、AVIF、JPEG XL 等格式；视频支持 MP4、MOV、MKV、WebM，内置 FFmpeg，可用硬件 VideoToolbox（H.264/HEVC）和软件 x264、x265、AV1 编码。它基于 Tauri 2 + Rust 构建，所有工具随应用内置，不需要用户额外安装命令行工具。适合博客作者、前端开发者、设计师、运营和独立开发者批量处理素材。

## 核心卖点

- 本地运行，素材不上云，断网也能用
- 免费开源，MIT 协议
- 图片和视频共用一个队列，拖入即自动识别媒体类型
- 内置图片压缩 7 件套和 FFmpeg，开箱即用
- 视频支持硬件 H.264/HEVC 与 x264、x265、AV1 软件编码
- 支持压缩前后滑动对比；视频还有同帧对比
- 结果必须有效、可播放且更小才算成功，不做假压缩
- HDR / 10bit / Alpha 等有风险的输入会被拒绝并解释，绝不静默处理
- 覆盖原文件前自动备份，可一键恢复
- Tauri 2 + Rust 构建，无前端框架依赖

## 推荐发布顺序

1. 先更新 GitHub README，确保仓库首页能承接流量
2. 发布中文博客文章，使用 `blog.md`
3. 发 V2EX「分享创造」
4. 发掘金技术文章，重点讲 Tauri + Rust + FFmpeg 管线与安全事务
5. 投稿少数派，重点讲本地压缩、隐私和效率
6. 准备英文版后发 Product Hunt、Reddit、Hacker News

## V2EX 文案

标题：

```text
我做了一个开源本地的图片 / 视频压缩工具：Sizqo
```

正文：

```text
最近做了一个小工具：Sizqo。

它是一个免费开源、本地运行的图片 / 视频压缩工具。主要解决几个痛点：

- 在线压缩要上传素材，处理客户素材、私人视频时不放心
- 命令行工具效果好，但 FFmpeg 参数对很多人来说像天书
- 桌面软件有些比较重，或者批量处理和对比体验不够顺手

Sizqo 支持拖入图片、视频或整个文件夹，自动识别媒体类型：

- 图片：批量压缩 PNG、JPG、GIF、WebP、BMP，可输出 WebP、AVIF、JPEG XL
- 视频：输出 MP4、MOV、MKV、WebM，硬件 VideoToolbox（H.264/HEVC）优先，软件 x264、x265、AV1 可选；音轨、字幕、章节要么保留要么明确告知

技术上用 Tauri 2 + Rust 做后端，内置 FFmpeg 和 pngquant、oxipng、mozjpeg、gifsicle、cwebp、avifenc。用户下载安装后就能用，不需要自己装 Homebrew 或背参数。

几个设计原则：

- 本地运行，素材不上云
- 批量拖拽，递归处理文件夹
- 压缩前后滑动对比，视频支持同帧对比
- 覆盖原文件前自动备份，可一键恢复
- 输出必须有效、可播放且更小才算成功，否则保留原文件
- HDR / 10bit / Alpha 等没把握的输入会被拒绝并解释，不静默降级
- MIT 开源

GitHub:
https://github.com/misswell/Sizqo

欢迎试用，也欢迎提 Issue / PR。
```

标签：

```text
分享创造, macOS, Tauri, Rust, FFmpeg, 视频压缩, 图片压缩
```

## 掘金文章角度

标题备选：

```text
用 Tauri + Rust 做了一个本地图片 / 视频压缩工具
```

```text
我把 FFmpeg 和 7 个图片压缩 CLI 打包进了一个 Tauri 桌面应用
```

文章结构：

```text
1. 为什么做：在线工具隐私、FFmpeg 参数门槛、桌面工具体积
2. 产品形态：一个队列吃图片和视频、批量、对比、恢复
3. 技术架构：Tauri 2 + Rust + 原生 HTML/CSS/JS
4. 图片引擎：pngquant、oxipng、mozjpeg、gifsicle、cwebp、avifenc
5. 视频管线：ffprobe 探测 → 能力矩阵 → 类型化 EncodePlan → 流式进度 → 全量解码校验 → 安全事务提交
6. 安全细节：输出比源大算 skippedLarger、崩溃恢复的预提交日志、备份绝不进临时目录
7. GitHub 地址和下载
```

## 少数派投稿角度

标题备选：

```text
Sizqo：让图片和视频压缩回到本地的一款轻量开源工具
```

导语：

```text
压缩本来应该是一件很轻的事：打开应用，拖入文件，得到更小的文件。Sizqo 是一个免费开源、本地运行的图片 / 视频压缩工具，适合处理博客配图、产品截图、设计稿、录屏和相机素材。
```

重点：

```text
少讲底层实现，多讲使用场景：隐私、本地、批量、对比、恢复，以及"不缩小就如实说"的结果策略。
```

## Product Hunt 文案

Name:

```text
Sizqo
```

Tagline:

```text
Open-source local image & video compressor
```

Description:

```text
Sizqo is a free, open-source desktop app for local media compression. Drop in images or videos, compress them on your machine, compare before and after, and keep your originals protected. Images export to WebP, AVIF, and more; videos encode with hardware VideoToolbox or x264/x265/AV1 through bundled FFmpeg. Built with Tauri 2 and Rust.
```

First comment:

```text
Hi Product Hunt,

I built Sizqo because compression should feel simple without sending private media to a remote server.

It runs locally, handles images and videos in one queue, includes compression engines out of the box, and refuses to fake results: an output only counts as compressed when it is valid, playable, and actually smaller. Risky inputs (HDR, 10-bit, alpha) are rejected with an explanation instead of silently tone-mapped.

It is built with Tauri 2 + Rust and released as an MIT open-source project.

I would love feedback from designers, developers, bloggers, and anyone who handles lots of screenshots, recordings, or website assets.
```

Topics:

```text
Open Source, Developer Tools, Design Tools, Productivity, Mac
```

## Reddit 文案

Subreddits:

```text
r/opensource
r/webdev
r/rust
r/macapps
r/SideProject
```

Post:

```text
I built Sizqo, a free open-source desktop app for local image AND video compression.

The idea is simple: drop in images, videos, or folders, compress everything on your machine, and compare the original vs compressed result. No upload, no external service, no CLI setup for the user.

Images support PNG, JPG, GIF, WebP, BMP and can output WebP, AVIF, JPEG XL, JPEG, and PNG. Videos output MP4, MOV, MKV, or WebM with hardware VideoToolbox (H.264/HEVC) or software x264/x265/SVT-AV1 through bundled FFmpeg. Audio tracks, subtitles, chapters, and attachments are preserved or explicitly dropped — never silently lost.

One rule I care about: an output only counts as success when it is valid, playable, and smaller than the source; otherwise the task reports "skippedLarger" and your original stays untouched.

Built with Tauri 2 + Rust.

GitHub:
https://github.com/misswell/Sizqo

Feedback is very welcome.
```

## GitHub Release 文案

标题：

```text
Sizqo <version>
```

正文：

```text
Sizqo is a free open-source local image and video compressor built with Tauri 2 and Rust.

Highlights:

- Compress images (PNG, JPG, GIF, WebP, BMP) and videos (MP4, MOV, MKV, WebM) locally
- Video encoders: hardware VideoToolbox H.264/HEVC, software x264, x265, SVT-AV1
- One queue for both media types with per-task progress and cancellation
- Compare images side by side and compare video frames at any timestamp
- Honest results: larger outputs are reported as skipped, never faked as success
- Backup before replace with one-click restore and crash-safe journals
- Built-in compression tools, no extra CLI setup required

Download the macOS build for your architecture below.
```

## 社交媒体短文案

中文：

```text
做了一个免费开源的本地图片 / 视频压缩工具：Sizqo。

拖入图片、视频或文件夹，它在电脑上批量压缩，素材不上传。图片支持 PNG/JPG/GIF/WebP/BMP，可输出 WebP、AVIF；视频输出 MP4/MOV/MKV/WebM，硬件编码优先，x264、x265、AV1 可选。Tauri + Rust，引擎内置，开箱即用。

GitHub: https://github.com/misswell/Sizqo
```

英文：

```text
I built Sizqo, a free open-source desktop app for local image & video compression.

Drop in images, videos, or folders — everything is compressed on your machine. Video encoding uses hardware VideoToolbox or bundled x264/x265/AV1, and results only count when they are valid and actually smaller.

Built with Tauri + Rust.

GitHub: https://github.com/misswell/Sizqo
```

## 截图使用建议

- `assets/banner.png`：博客头图、社交媒体配图、GitHub README 首图
- `assets/screenshot-light.png`：产品主界面
- `assets/screenshot-dark.png`：暗黑模式展示
- `assets/compare.png`：重点展示压缩前后对比功能
- `assets/icon.png`：头像、Product Hunt 图标、帖子缩略图

## 发布前检查清单

- GitHub Releases 里有当前版本安装包
- README 首屏能看到截图、下载地址和核心卖点
- Release 页面说明签名与公证状态，以及 macOS 的打开方式
- 博客文章中图片路径在发布平台可访问
- Product Hunt 需要准备 Logo、Gallery 图片和英文描述
- Reddit / Hacker News 发帖时避免过度营销，重点讲开源和实现细节
- V2EX 发帖后及时回复反馈，收集 bug 和安装问题
