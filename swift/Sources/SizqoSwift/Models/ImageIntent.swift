import Foundation

/// 图片意图：core/intent-model.md「Intent model / Image」表的唯一实现。
/// 意图 → 引擎参数由 `apply(to:)` 编译；改动任何原始参数会把意图切成 `.custom`。
enum ImageIntent: String, CaseIterable {
    case losslessFirst
    case balanced
    case asSmallAsPossible
    case webSize
    case targetSize
    case custom

    /// 契约里的标签；`balanced` 的「（默认）」是说明，不是名字的一部分。
    var label: String {
        switch self {
        case .losslessFirst: return "肉眼无差优先"
        case .balanced: return "均衡"
        case .asSmallAsPossible: return "尽量变小"
        case .webSize: return "网页尺寸"
        case .targetSize: return "指定体积"
        case .custom: return "自定义"
        }
    }

    /// 与 Direct 意图表的 note 同一批事实：档位、是否动尺寸、候选搜索会不会换格式。
    var helpText: String {
        switch self {
        case .losslessFirst: return "质量 90，不改变像素尺寸；候选搜索挑最小，PNG 可能被换成更小的 WebP。"
        case .balanced: return "质量 75，不改变像素尺寸；候选搜索挑最小，PNG 可能被换成更小的 WebP。默认。"
        case .asSmallAsPossible: return "质量 60，候选搜索取最小；PNG 会同时试 WebP 并按更小者输出，JPG 只压 JPG。"
        case .webSize: return "最长边压到 2560 px 以内（只缩不放），质量 72，格式保持不变。"
        case .targetSize: return "在体积预算内自动组合尺寸与质量（先降质量 80/65/50，再压尺寸 2560/1920/1280，最多 6 次编码），取达标的最小结果；达不到就报最接近的结果，不谎报成功。"
        case .custom: return "直接用下面这些参数，意图不再介入：怎么写就怎么发，PNG 不会被换成更小的 WebP。改动任一参数即切到这里。"
        }
    }

    /// 诚实规则 3：高级路径重编码像素，EXIF/GPS 进不了输出。这句话必须在按下开始
    /// 之前说（与 Direct 意图表的 warning 同一件事），不能等压完再补。
    var metadataNote: String {
        "高级路径重编码像素，EXIF 与 GPS 不会带入输出；要保留元数据请改用系统转换。"
    }

    /// 意图决定的智能路由（真实引擎行为：PNG 同时试 PNG 与 WebP 候选取更小）。
    var usesSmartRouting: Bool {
        switch self {
        case .losslessFirst, .balanced, .asSmallAsPossible: return true
        default: return false
        }
    }

    /// targetSize 未给预算时的起步值：512 KiB。
    static let defaultBudgetBytes: Int64 = 512 * 1024

    /// 把意图编译进一份原始参数（outputMode / suffix / dir / roots 等落盘语义原样保留）。
    /// `targetSize` 的预算是意图输入，保留用户已填的 `maxOutputBytes`。
    /// 编译结果必须与 Direct / App Store 的意图表逐字一致：三轨的候选集合由引擎决定
    /// （PNG 试 PNG 与 WebP 取更小，JPG 只试 JPG），意图不许在某一轨偷偷加候选。
    func apply(to base: CompressOptions) -> CompressOptions {
        var o = base
        switch self {
        case .losslessFirst:
            o.quality = 90; o.outputFormat = .original
            o.maxDimensionPx = nil; o.maxOutputBytes = nil
        case .balanced:
            o.quality = 75; o.outputFormat = .original
            o.maxDimensionPx = nil; o.maxOutputBytes = nil
        case .asSmallAsPossible:
            o.quality = 60; o.outputFormat = .original
            o.maxDimensionPx = nil; o.maxOutputBytes = nil
        case .webSize:
            o.quality = 72; o.outputFormat = .original
            o.maxDimensionPx = 2560; o.maxOutputBytes = nil
        case .targetSize:
            o.quality = 0; o.outputFormat = .original
            o.maxDimensionPx = nil
            o.maxOutputBytes = base.maxOutputBytes ?? ImageIntent.defaultBudgetBytes
        case .custom:
            break
        }
        return o
    }

    /// 屏幕上「本轮将发送的请求」摘要（契约：改成 custom 后结果要看得懂）。
    func requestSummary(base: CompressOptions) -> String {
        let o = apply(to: base)
        if o.processingMode == .system { return "系统转换 · \(o.outputFormat.label) · \(o.systemImageSize.label)" }
        var parts: [String] = [label]
        if let b = o.maxOutputBytes {
            parts.append("预算 \(CompressResult.formatBytesJS(b))")
        } else {
            parts.append("质量 \(o.quality)")
        }
        if usesSmartRouting && o.outputFormat == .original { parts.append("候选取最小") }
        if o.outputFormat != .original { parts.append("转 \(o.outputFormat.label)") }
        if let d = o.maxDimensionPx { parts.append("最长边 ≤ \(d) px") }
        parts.append(o.outputMode.label)
        return parts.joined(separator: " · ")
    }
}
