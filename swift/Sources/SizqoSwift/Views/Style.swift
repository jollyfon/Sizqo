import SwiftUI

// MARK: - 设计 token（明暗两套均由语义色 / primary 混合自动适配，不写死十六进制）

/// 颜色：全部走 AppKit 语义色或 primary 蒙层，浅色 / 深色自动正确。
enum SizqoPalette {
    /// 窗口 / rail 底
    static let window = Color(nsColor: .windowBackgroundColor)
    /// 内容面板 / 卡片底
    static let surface = Color(nsColor: .controlBackgroundColor)
    /// 比 surface 更沉一层的凹陷区（列表条、统计条）
    static let surfaceSunken = Color.primary.opacity(0.04)
    /// 选中 / hover 强调底
    static let emphasis = Color.primary.opacity(0.08)
    static let emphasisSoft = Color.primary.opacity(0.05)
    /// 发丝描边
    static let hairline = Color(nsColor: .separatorColor).opacity(0.6)
    /// 三级文字
    static let tertiaryText = Color(nsColor: .tertiaryLabelColor)
    /// 状态色（与队列行既有语义一致）
    static let success = Color.green
    static let danger = Color.red
    static let caution = Color.orange
    static let running = Color.accentColor
    static let accentSoft = Color.accentColor.opacity(0.12)
}

/// 间距刻度（4 的基格 + 既有 14px 行内边距档）
enum SizqoSpacing {
    static let xxs: CGFloat = 2
    static let xs: CGFloat = 4
    static let sm: CGFloat = 8
    static let md: CGFloat = 12
    static let lg: CGFloat = 16
    static let xl: CGFloat = 20
    static let xxl: CGFloat = 28
    /// 卡片 / 面板内水平留白（原 .queue-header 14px）
    static let rowX: CGFloat = 14
}

/// 圆角刻度
enum SizqoRadius {
    static let control: CGFloat = 6
    static let card: CGFloat = 8
    static let chip: CGFloat = 10
    static let panel: CGFloat = 12
    static let pill: CGFloat = 999
}

/// 字号梯（沿用既有 10–18 密度，收成一条明确的 ramp）
enum SizqoType {
    static let micro = Font.system(size: 10)
    static let microBold = Font.system(size: 10, weight: .semibold)
    static let caption = Font.system(size: 11)
    static let captionMedium = Font.system(size: 11, weight: .medium)
    static let body = Font.system(size: 12)
    static let bodyMedium = Font.system(size: 12, weight: .medium)
    static let control = Font.system(size: 13, weight: .semibold)
    static let statValue = Font.system(size: 14, weight: .bold, design: .rounded)
    static let headline = Font.system(size: 15, weight: .semibold)
    static func mono(_ size: CGFloat) -> Font { Font.system(size: size, design: .monospaced) }
    static func digit(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        Font.system(size: size, weight: weight).monospacedDigit()
    }
}

// MARK: - 设计规范（对齐 frontend/style.css 的 Tauri 设计基准；新代码优先用上面的 token）

enum AppMetrics {
    /// Tauri --titlebar-height: 42px
    static let titleBarHeight: CGFloat = 42
    /// Tauri --radius: 8px（卡片圆角）
    static let cardRadius = SizqoRadius.card
    /// Tauri --radius-sm: 6px（按钮/行圆角）
    static let smallRadius = SizqoRadius.control
    /// Tauri .container: padding 12px 16px 16px
    static let containerPadding = EdgeInsets(top: 12, leading: 16, bottom: 16, trailing: 16)
    /// 卡片内水平留白（.settings-header / .queue-header 等的 14px）
    static let sectionHPadding = SizqoSpacing.rowX
    /// Tauri 卡片间距（.settings-panel/.queue-panel 的 margin-bottom: 12px）
    static let cardSpacing = SizqoSpacing.md
    /// 双栏工作区的详情栏宽度
    static let detailColumnWidth: CGFloat = 340
    /// rail 宽度（图标 + 两字标签）
    static let railWidth: CGFloat = 76
}

// MARK: - 卡片容器（.settings-panel / .queue-panel 底板）

struct CardBackground: ViewModifier {
    var cornerRadius: CGFloat = AppMetrics.cardRadius

    func body(content: Content) -> some View {
        content
            .background(
                RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                    .fill(SizqoPalette.surface)
            )
            .overlay(
                RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                    .strokeBorder(SizqoPalette.hairline, lineWidth: 0.5)
            )
            .shadow(color: Color.black.opacity(0.07), radius: 1.5, y: 1)
    }
}

extension View {
    func cardStyle(cornerRadius: CGFloat = AppMetrics.cardRadius) -> some View {
        modifier(CardBackground(cornerRadius: cornerRadius))
    }

    /// 面板底：无描边无投影的纯表面（三栏分区用）
    func panelSurface() -> some View {
        background(SizqoPalette.surface)
    }
}

// MARK: - 半透明发丝分隔线（0.5px border-light）

struct HairlineDivider: View {
    var body: some View {
        Divider()
            .opacity(0.5)
    }
}

// MARK: - 按钮样式

/// 标题栏右侧 26pt 小按钮（.tb-btn）
struct TitleBarButtonStyle: ButtonStyle {
    /// 当前页高亮（历史 / 设置入口与 .tb-btn.active 对齐）
    var active = false
    @State private var hovering = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 13))
            .foregroundColor(active ? .accentColor : (hovering ? .primary : .secondary))
            .frame(width: 26, height: 26)
            .background(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .fill(Color.primary.opacity(active ? 0.10 : (hovering ? 0.08 : 0)))
            )
            .onHover { hovering = $0 }
    }
}

/// 队列行内 22pt 图标按钮（.queue-action-btn）
struct RowActionButtonStyle: ButtonStyle {
    @State private var hovering = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 11))
            .frame(width: 22, height: 22)
            .background(
                RoundedRectangle(cornerRadius: 4, style: .continuous)
                    .fill(Color.primary.opacity(hovering ? 0.08 : 0))
            )
            .foregroundColor(hovering ? .primary : .secondary)
            .onHover { hovering = $0 }
    }
}

/// 主按钮（.btn-primary：12px/500，padding 6×16，radius 6）
struct PrimaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 12, weight: .medium))
            .padding(.horizontal, 16)
            .padding(.vertical, 6)
            .background(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .fill(Color.accentColor.opacity(configuration.isPressed ? 0.8 : 1))
            )
            .foregroundColor(.white)
    }
}

/// 次按钮（.btn-secondary）
struct SecondaryButtonStyle: ButtonStyle {
    @State private var hovering = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 12, weight: .medium))
            .padding(.horizontal, 16)
            .padding(.vertical, 6)
            .background(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .fill(Color(nsColor: .controlBackgroundColor))
            )
            .overlay(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .strokeBorder(Color(nsColor: .separatorColor).opacity(0.8), lineWidth: 0.5)
            )
            .opacity(configuration.isPressed ? 0.7 : (hovering ? 1.0 : 0.95))
            .onHover { hovering = $0 }
    }
}

/// 幽灵按钮（.btn-ghost + .btn-small：强调色文字，透明底，padding 4×10，11px）
struct GhostButtonStyle: ButtonStyle {
    @State private var hovering = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 11))
            .foregroundColor(.accentColor)
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .fill(Color.primary.opacity(hovering ? 0.06 : 0))
            )
            .onHover { hovering = $0 }
    }
}

/// 小号带边框按钮（.btn.btn-small：11px，padding 4×10，与列表行同密度）
struct SmallButtonStyle: ButtonStyle {
    @State private var hovering = false

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 11, weight: .medium))
            .foregroundColor(hovering ? .primary : .secondary)
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .fill(Color(nsColor: .controlBackgroundColor))
            )
            .overlay(
                RoundedRectangle(cornerRadius: AppMetrics.smallRadius, style: .continuous)
                    .strokeBorder(Color(nsColor: .separatorColor).opacity(0.8), lineWidth: 0.5)
            )
            .opacity(configuration.isPressed ? 0.7 : 1)
            .onHover { hovering = $0 }
    }
}

// MARK: - 页面头（列标题；rail 是唯一导航，这里不再放返回按钮）

struct ViewHeaderView<Trailing: View>: View {
    let title: String
    /// SizqoIcon glyph 名字（与前端 sprite 同一套 id），不是 SF Symbol。
    let icon: String
    let trailing: Trailing

    init(title: String, icon: String, @ViewBuilder trailing: () -> Trailing) {
        self.title = title
        self.icon = icon
        self.trailing = trailing()
    }

    var body: some View {
        HStack(spacing: 8) {
            HStack(spacing: 5) {
                SizqoIcon(name: icon, size: 15, color: .accentColor)
                Text(title).font(SizqoType.control)
            }
            .foregroundColor(.primary)

            Spacer()
            trailing
        }
        .padding(.horizontal, SizqoSpacing.rowX)
        .padding(.vertical, 10)
    }
}

extension ViewHeaderView where Trailing == EmptyView {
    init(title: String, icon: String) {
        self.init(title: title, icon: icon, trailing: { EmptyView() })
    }
}

// MARK: - 设置行（.settings-row：label 12px/500 min-width 72，行内边距 8px，底部发丝线）

struct SettingsRow<Content: View>: View {
    let label: String
    var showSeparator = true
    @ViewBuilder let content: () -> Content

    var body: some View {
        HStack(alignment: .center, spacing: 8) {
            Text(label)
                .font(.system(size: 12, weight: .medium))
                // Tauri label 为 min-width: 72px，长标签自然撑开不换行
                .frame(minWidth: 72, alignment: .leading)
            content()
            Spacer(minLength: 0)
        }
        .padding(.vertical, 8)
        .overlay(alignment: .bottom) {
            if showSeparator { HairlineDivider() }
        }
    }
}
