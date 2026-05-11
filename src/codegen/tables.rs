//! 静态查找表和常量定义
//!
//! 本模块包含所有编译期常量表和 O(1) 查找函数，不依赖其他 codegen 子模块。
//!
//! 优化策略：所有查找均使用 match 语句，
//! 编译器可生成跳转表或 trie 结构，实现 O(1) 查找。

/// 查找颜色值（O(1) match 查找）
///
/// 完整 Tailwind 色板，编译器为 match 生成高效跳转表/trie。
pub(crate) fn lookup_color(name: &str) -> Option<u32> {
    match name {
        // amber
        "amber_50" => Some(0xfffbeb),
        "amber_100" => Some(0xfef3c7),
        "amber_200" => Some(0xfde68a),
        "amber_300" => Some(0xfcd34d),
        "amber_400" => Some(0xfbbf24),
        "amber_500" => Some(0xf59e0b),
        "amber_600" => Some(0xd97706),
        "amber_700" => Some(0xb45309),
        "amber_800" => Some(0x92400e),
        "amber_900" => Some(0x78350f),
        "amber_950" => Some(0x451a03),
        // blue
        "blue_50" => Some(0xeff6ff),
        "blue_100" => Some(0xdbeafe),
        "blue_200" => Some(0xbfdbfe),
        "blue_300" => Some(0x93c5fd),
        "blue_400" => Some(0x60a5fa),
        "blue_500" => Some(0x3b82f6),
        "blue_600" => Some(0x2563eb),
        "blue_700" => Some(0x1d4ed8),
        "blue_800" => Some(0x1e40af),
        "blue_900" => Some(0x1e3a8a),
        "blue_950" => Some(0x172554),
        // cyan
        "cyan_50" => Some(0xecfeff),
        "cyan_100" => Some(0xcffafe),
        "cyan_200" => Some(0xa5f3fc),
        "cyan_300" => Some(0x67e8f9),
        "cyan_400" => Some(0x22d3ee),
        "cyan_500" => Some(0x06b6d4),
        "cyan_600" => Some(0x0891b2),
        "cyan_700" => Some(0x0e7490),
        "cyan_800" => Some(0x155e75),
        "cyan_900" => Some(0x164e63),
        "cyan_950" => Some(0x083344),
        // emerald
        "emerald_50" => Some(0xecfdf5),
        "emerald_100" => Some(0xd1fae5),
        "emerald_200" => Some(0xa7f3d0),
        "emerald_300" => Some(0x6ee7b7),
        "emerald_400" => Some(0x34d399),
        "emerald_500" => Some(0x10b981),
        "emerald_600" => Some(0x059669),
        "emerald_700" => Some(0x047857),
        "emerald_800" => Some(0x065f46),
        "emerald_900" => Some(0x064e3b),
        "emerald_950" => Some(0x022c22),
        // fuchsia
        "fuchsia_50" => Some(0xfdf4ff),
        "fuchsia_100" => Some(0xfae8ff),
        "fuchsia_200" => Some(0xf5d0fe),
        "fuchsia_300" => Some(0xf0abfc),
        "fuchsia_400" => Some(0xe879f9),
        "fuchsia_500" => Some(0xd946ef),
        "fuchsia_600" => Some(0xc026d3),
        "fuchsia_700" => Some(0xa21caf),
        "fuchsia_800" => Some(0x86198f),
        "fuchsia_900" => Some(0x701a75),
        "fuchsia_950" => Some(0x4a044e),
        // gray
        "gray_50" => Some(0xf9fafb),
        "gray_100" => Some(0xf3f4f6),
        "gray_200" => Some(0xe5e7eb),
        "gray_300" => Some(0xd1d5db),
        "gray_400" => Some(0x9ca3af),
        "gray_500" => Some(0x6b7280),
        "gray_600" => Some(0x4b5563),
        "gray_700" => Some(0x374151),
        "gray_800" => Some(0x1f2937),
        "gray_900" => Some(0x111827),
        "gray_950" => Some(0x030712),
        // green
        "green_50" => Some(0xf0fdf4),
        "green_100" => Some(0xdcfce7),
        "green_200" => Some(0xbbf7d0),
        "green_300" => Some(0x86efac),
        "green_400" => Some(0x4ade80),
        "green_500" => Some(0x22c55e),
        "green_600" => Some(0x16a34a),
        "green_700" => Some(0x15803d),
        "green_800" => Some(0x166534),
        "green_900" => Some(0x14532d),
        "green_950" => Some(0x052e16),
        // indigo
        "indigo_50" => Some(0xeef2ff),
        "indigo_100" => Some(0xe0e7ff),
        "indigo_200" => Some(0xc7d2fe),
        "indigo_300" => Some(0xa5b4fc),
        "indigo_400" => Some(0x818cf8),
        "indigo_500" => Some(0x6366f1),
        "indigo_600" => Some(0x4f46e5),
        "indigo_700" => Some(0x4338ca),
        "indigo_800" => Some(0x3730a3),
        "indigo_900" => Some(0x312e81),
        "indigo_950" => Some(0x1e1b4b),
        // lime
        "lime_50" => Some(0xf7fee7),
        "lime_100" => Some(0xecfccb),
        "lime_200" => Some(0xd9f99d),
        "lime_300" => Some(0xbef264),
        "lime_400" => Some(0xa3e635),
        "lime_500" => Some(0x84cc16),
        "lime_600" => Some(0x65a30d),
        "lime_700" => Some(0x4d7c0f),
        "lime_800" => Some(0x3f6212),
        "lime_900" => Some(0x365314),
        "lime_950" => Some(0x1a2e05),
        // neutral
        "neutral_50" => Some(0xfafafa),
        "neutral_100" => Some(0xf5f5f5),
        "neutral_200" => Some(0xe5e5e5),
        "neutral_300" => Some(0xd4d4d4),
        "neutral_400" => Some(0xa3a3a3),
        "neutral_500" => Some(0x737373),
        "neutral_600" => Some(0x525252),
        "neutral_700" => Some(0x404040),
        "neutral_800" => Some(0x262626),
        "neutral_900" => Some(0x171717),
        "neutral_950" => Some(0x0a0a0a),
        // orange
        "orange_50" => Some(0xfff7ed),
        "orange_100" => Some(0xffedd5),
        "orange_200" => Some(0xfed7aa),
        "orange_300" => Some(0xfdba74),
        "orange_400" => Some(0xfb923c),
        "orange_500" => Some(0xf97316),
        "orange_600" => Some(0xea580c),
        "orange_700" => Some(0xc2410c),
        "orange_800" => Some(0x9a3412),
        "orange_900" => Some(0x7c2d12),
        "orange_950" => Some(0x431407),
        // pink
        "pink_50" => Some(0xfdf2f8),
        "pink_100" => Some(0xfce7f3),
        "pink_200" => Some(0xfbcfe8),
        "pink_300" => Some(0xf9a8d4),
        "pink_400" => Some(0xf472b6),
        "pink_500" => Some(0xec4899),
        "pink_600" => Some(0xdb2777),
        "pink_700" => Some(0xbe185d),
        "pink_800" => Some(0x9d174d),
        "pink_900" => Some(0x831843),
        "pink_950" => Some(0x500724),
        // purple
        "purple_50" => Some(0xfaf5ff),
        "purple_100" => Some(0xf3e8ff),
        "purple_200" => Some(0xe9d5ff),
        "purple_300" => Some(0xd8b4fe),
        "purple_400" => Some(0xc084fc),
        "purple_500" => Some(0xa855f7),
        "purple_600" => Some(0x9333ea),
        "purple_700" => Some(0x7e22ce),
        "purple_800" => Some(0x6b21a8),
        "purple_900" => Some(0x581c87),
        "purple_950" => Some(0x3b0764),
        // red
        "red_50" => Some(0xfef2f2),
        "red_100" => Some(0xfee2e2),
        "red_200" => Some(0xfecaca),
        "red_300" => Some(0xfca5a5),
        "red_400" => Some(0xf87171),
        "red_500" => Some(0xef4444),
        "red_600" => Some(0xdc2626),
        "red_700" => Some(0xb91c1c),
        "red_800" => Some(0x991b1b),
        "red_900" => Some(0x7f1d1d),
        "red_950" => Some(0x450a0a),
        // rose
        "rose_50" => Some(0xfff1f2),
        "rose_100" => Some(0xffe4e6),
        "rose_200" => Some(0xfecdd3),
        "rose_300" => Some(0xfda4af),
        "rose_400" => Some(0xfb7185),
        "rose_500" => Some(0xf43f5e),
        "rose_600" => Some(0xe11d48),
        "rose_700" => Some(0xbe123c),
        "rose_800" => Some(0x9f1239),
        "rose_900" => Some(0x881337),
        "rose_950" => Some(0x4c0519),
        // sky
        "sky_50" => Some(0xf0f9ff),
        "sky_100" => Some(0xe0f2fe),
        "sky_200" => Some(0xbae6fd),
        "sky_300" => Some(0x7dd3fc),
        "sky_400" => Some(0x38bdf8),
        "sky_500" => Some(0x0ea5e9),
        "sky_600" => Some(0x0284c7),
        "sky_700" => Some(0x0369a1),
        "sky_800" => Some(0x075985),
        "sky_900" => Some(0x0c4a6e),
        "sky_950" => Some(0x082f49),
        // slate
        "slate_50" => Some(0xf8fafc),
        "slate_100" => Some(0xf1f5f9),
        "slate_200" => Some(0xe2e8f0),
        "slate_300" => Some(0xcbd5e1),
        "slate_400" => Some(0x94a3b8),
        "slate_500" => Some(0x64748b),
        "slate_600" => Some(0x475569),
        "slate_700" => Some(0x334155),
        "slate_800" => Some(0x1e293b),
        "slate_900" => Some(0x0f172a),
        "slate_950" => Some(0x020617),
        // stone
        "stone_50" => Some(0xfafaf9),
        "stone_100" => Some(0xf5f5f4),
        "stone_200" => Some(0xe7e5e4),
        "stone_300" => Some(0xd6d3d1),
        "stone_400" => Some(0xa8a29e),
        "stone_500" => Some(0x78716c),
        "stone_600" => Some(0x57534e),
        "stone_700" => Some(0x44403c),
        "stone_800" => Some(0x292524),
        "stone_900" => Some(0x1c1917),
        "stone_950" => Some(0x0c0a09),
        // teal
        "teal_50" => Some(0xf0fdfa),
        "teal_100" => Some(0xccfbf1),
        "teal_200" => Some(0x99f6e4),
        "teal_300" => Some(0x5eead4),
        "teal_400" => Some(0x2dd4bf),
        "teal_500" => Some(0x14b8a6),
        "teal_600" => Some(0x0d9488),
        "teal_700" => Some(0x0f766e),
        "teal_800" => Some(0x115e59),
        "teal_900" => Some(0x134e4a),
        "teal_950" => Some(0x042f2e),
        // violet
        "violet_50" => Some(0xf5f3ff),
        "violet_100" => Some(0xede9fe),
        "violet_200" => Some(0xddd6fe),
        "violet_300" => Some(0xc4b5fd),
        "violet_400" => Some(0xa78bfa),
        "violet_500" => Some(0x8b5cf6),
        "violet_600" => Some(0x7c3aed),
        "violet_700" => Some(0x6d28d9),
        "violet_800" => Some(0x5b21b6),
        "violet_900" => Some(0x4c1d95),
        "violet_950" => Some(0x2e1065),
        // yellow
        "yellow_50" => Some(0xfefce8),
        "yellow_100" => Some(0xfef9c3),
        "yellow_200" => Some(0xfef08a),
        "yellow_300" => Some(0xfde047),
        "yellow_400" => Some(0xfacc15),
        "yellow_500" => Some(0xeab308),
        "yellow_600" => Some(0xca8a04),
        "yellow_700" => Some(0xa16207),
        "yellow_800" => Some(0x854d0e),
        "yellow_900" => Some(0x713f12),
        "yellow_950" => Some(0x422006),
        // zinc
        "zinc_50" => Some(0xfafafa),
        "zinc_100" => Some(0xf4f4f5),
        "zinc_200" => Some(0xe4e4e7),
        "zinc_300" => Some(0xd4d4d8),
        "zinc_400" => Some(0xa1a1aa),
        "zinc_500" => Some(0x71717a),
        "zinc_600" => Some(0x52525b),
        "zinc_700" => Some(0x3f3f46),
        "zinc_800" => Some(0x27272a),
        "zinc_900" => Some(0x18181b),
        "zinc_950" => Some(0x09090b),
        // black & white
        "black" => Some(0x000000),
        "white" => Some(0xffffff),
        _ => None,
    }
}

/// 查找属性的 GPUI 方法名（事件处理器 + camelCase 属性映射）
///
/// 使用 match 语句替代线性扫描，编译器生成高效跳转表。
/// 合并了原 EVENT_HANDLERS 和 ATTRIBUTE_NAME_MAP 两张表。
pub(crate) fn lookup_attr_method(name: &str) -> Option<&'static str> {
    match name {
        // 事件处理器（camelCase 和 snake_case 均支持）
        "onClick" | "on_click" => Some("on_click"),
        "onMouseDown" | "on_mouse_down" => Some("on_mouse_down"),
        "onMouseUp" | "on_mouse_up" => Some("on_mouse_up"),
        "onMouseMove" | "on_mouse_move" => Some("on_mouse_move"),
        "onMouseDownOut" | "on_mouse_down_out" => Some("on_mouse_down_out"),
        "onMouseUpOut" | "on_mouse_up_out" => Some("on_mouse_up_out"),
        "onAnyMouseDown" | "on_any_mouse_down" => Some("on_any_mouse_down"),
        "onAnyMouseUp" | "on_any_mouse_up" => Some("on_any_mouse_up"),
        "onKeyDown" | "on_key_down" => Some("on_key_down"),
        "onKeyUp" | "on_key_up" => Some("on_key_up"),
        "onModifiersChanged" | "on_modifiers_changed" => Some("on_modifiers_changed"),
        "onHover" | "on_hover" => Some("on_hover"),
        "onScrollWheel" | "on_scroll_wheel" => Some("on_scroll_wheel"),
        "onDrag" | "on_drag" => Some("on_drag"),
        "onDragMove" | "on_drag_move" => Some("on_drag_move"),
        "onDrop" | "on_drop" => Some("on_drop"),
        "onAction" | "on_action" => Some("on_action"),
        "onBoxedAction" | "on_boxed_action" => Some("on_boxed_action"),
        // 捕获阶段事件处理器
        "captureAnyMouseDown" | "capture_any_mouse_down" => Some("capture_any_mouse_down"),
        "captureAnyMouseUp" | "capture_any_mouse_up" => Some("capture_any_mouse_up"),
        "captureKeyDown" | "capture_key_down" => Some("capture_key_down"),
        "captureKeyUp" | "capture_key_up" => Some("capture_key_up"),
        "captureAction" | "capture_action" => Some("capture_action"),
        // 属性名称映射（camelCase → snake_case，仅非恒等映射）
        "blockMouseExceptScroll" => Some("block_mouse_except_scroll"),
        "canDrop" => Some("can_drop"),
        "debugSelector" => Some("debug_selector"),
        "dragOver" => Some("drag_over"),
        "groupActive" => Some("group_active"),
        "groupDragOver" => Some("group_drag_over"),
        "groupHover" => Some("group_hover"),
        "inFocus" => Some("in_focus"),
        "keyContext" => Some("key_context"),
        "tabGroup" => Some("tab_group"),
        "tabIndex" => Some("tab_index"),
        "tabStop" => Some("tab_stop"),
        "trackFocus" => Some("track_focus"),
        "windowControlArea" => Some("window_control_area"),
        "anchorScroll" => Some("anchor_scroll"),
        "hoverableTooltip" => Some("hoverable_tooltip"),
        "overflowScroll" => Some("overflow_scroll"),
        "overflowXScroll" => Some("overflow_x_scroll"),
        "overflowYScroll" => Some("overflow_y_scroll"),
        "scrollbarWidth" => Some("scrollbar_width"),
        "trackScroll" => Some("track_scroll"),
        "width" => Some("w"),
        "height" => Some("h"),
        "minWidth" => Some("min_w"),
        "minHeight" => Some("min_h"),
        "maxWidth" => Some("max_w"),
        "maxHeight" => Some("max_h"),
        "gapX" => Some("gap_x"),
        "gapY" => Some("gap_y"),
        "flexBasis" => Some("flex_basis"),
        "fontSize" => Some("text_size"),
        "lineHeight" => Some("line_height"),
        "fontWeight" => Some("font_weight"),
        "textAlign" => Some("text_align"),
        "borderTop" => Some("border_t"),
        "borderBottom" => Some("border_b"),
        "borderLeft" => Some("border_l"),
        "borderRight" => Some("border_r"),
        "roundedTop" => Some("rounded_t"),
        "roundedBottom" => Some("rounded_b"),
        "roundedTopLeft" => Some("rounded_tl"),
        "roundedTopRight" => Some("rounded_tr"),
        "roundedBottomLeft" => Some("rounded_bl"),
        "roundedBottomRight" => Some("rounded_br"),
        "boxShadow" => Some("shadow"),
        // Grid 布局属性
        "gridCols" => Some("grid_cols"),
        "gridRows" => Some("grid_rows"),
        "colSpan" => Some("col_span"),
        "rowSpan" => Some("row_span"),
        "colStart" => Some("col_start"),
        "colEnd" => Some("col_end"),
        "rowStart" => Some("row_start"),
        "rowEnd" => Some("row_end"),
        _ => None,
    }
}

/// 查找 flag 属性的 GPUI 方法名。
///
/// 绝大多数 flag 属性可直接复用 value 属性映射；方向性 border 是例外：
/// `border_t={value}` 应调用 `.border_t(value)`，而 `border_t` flag 应调用
/// GPUI 0.2 的预设宽度方法 `.border_t_1()`。
pub(crate) fn lookup_attr_flag_method(name: &str) -> Option<&'static str> {
    match name {
        "flexGrow" => Some("flex_grow"),
        "flexShrink" => Some("flex_shrink"),
        "border_t" => Some("border_t_1"),
        "border_b" => Some("border_b_1"),
        "border_l" => Some("border_l_1"),
        "border_r" => Some("border_r_1"),
        "border_x" => Some("border_x_1"),
        "border_y" => Some("border_y_1"),
        _ => lookup_attr_method(name),
    }
}

/// 检查属性是否需要 stateful element（需要 `.id()`）
///
/// GPUI 0.2 将大多数事件放在 `InteractiveElement` 上，不要求 stateful ID。
/// 只有 `StatefulInteractiveElement` 方法需要先调用 `.id()`。
pub(crate) fn is_stateful_attr(name: &str) -> bool {
    if let Some(method) = lookup_attr_method(name) {
        return is_stateful_method(method);
    }

    is_stateful_method(name)
}

pub(crate) fn is_stateful_method(method: &str) -> bool {
    // `active` here is StatefulInteractiveElement::active (takes closure), not Styled::active.
    // hover/focus/group are Styled trait methods that don't need .id().
    matches!(
        method,
        "active"
            | "anchor_scroll"
            | "focusable"
            | "group_active"
            | "hoverable_tooltip"
            | "on_click"
            | "on_drag"
            | "on_hover"
            | "overflow_scroll"
            | "overflow_x_scroll"
            | "overflow_y_scroll"
            | "scrollbar_width"
            | "tooltip"
            | "track_scroll"
    )
}

pub(crate) fn is_multi_arg_method(method: &str) -> bool {
    matches!(
        method,
        "group_active"
            | "group_drag_over"
            | "group_hover"
            | "on_boxed_action"
            | "on_drag"
            | "on_mouse_down"
            | "on_mouse_up"
            | "on_mouse_up_out"
    )
}

/// 检查静态 class 是否会调用 `StatefulInteractiveElement` 方法。
pub(crate) fn is_stateful_class(class: &str) -> bool {
    matches!(
        class,
        "overflow-scroll" | "overflow-x-scroll" | "overflow-y-scroll"
    )
}

/// 查找间距/尺寸 class 前缀对应的 GPUI 方法名
///
/// 使用 match 替代原 SPACING_PATTERNS 数组的线性扫描（17 项 → O(1)）。
pub(crate) fn lookup_spacing_method(prefix: &str) -> Option<&'static str> {
    match prefix {
        "gap_" => Some("gap"),
        "gap_x_" => Some("gap_x"),
        "gap_y_" => Some("gap_y"),
        "p_" => Some("p"),
        "px_" => Some("px"),
        "py_" => Some("py"),
        "pt_" => Some("pt"),
        "pb_" => Some("pb"),
        "pl_" => Some("pl"),
        "pr_" => Some("pr"),
        "m_" => Some("m"),
        "mx_" => Some("mx"),
        "my_" => Some("my"),
        "mt_" => Some("mt"),
        "mb_" => Some("mb"),
        "ml_" => Some("ml"),
        "mr_" => Some("mr"),
        "w_" => Some("w"),
        "h_" => Some("h"),
        "size_" => Some("size"),
        "min_w_" => Some("min_w"),
        "max_w_" => Some("max_w"),
        "min_h_" => Some("min_h"),
        "max_h_" => Some("max_h"),
        _ => None,
    }
}

/// 完整 Tailwind 色系名列表（22 个），供 runtime.rs 和单元测试共享使用。
///
/// 单一数据源：修改此处即可同步影响动态 class 运行时匹配表和颜色覆盖率测试。
pub(crate) const COLOR_FAMILIES: &[&str] = &[
    "amber", "blue", "cyan", "emerald", "fuchsia", "gray", "green", "indigo", "lime", "neutral",
    "orange", "pink", "purple", "red", "rose", "sky", "slate", "stone", "teal", "violet", "yellow",
    "zinc",
];

/// 完整 Tailwind 色阶列表（11 个），供 runtime.rs 和单元测试共享使用。
pub(crate) const COLOR_SHADES: &[&str] = &[
    "50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950",
];

/// 检查是否是有效的文本大小名称
///
/// 使用 match 替代原 VALID_TEXT_SIZES 数组的 `.contains()` 线性扫描。
pub(crate) fn is_valid_text_size(size: &str) -> bool {
    matches!(size, "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl")
}

/// 查找标签默认样式（仅当元素带 `styled` 标志时使用）
///
/// 使用 match 替代原 TAG_DEFAULT_STYLES 数组的 `.iter().find()` 线性扫描。
pub(crate) fn lookup_tag_default(tag: &str) -> Option<&'static str> {
    match tag {
        "h1" => Some("text-3xl font-bold"),
        "h2" => Some("text-2xl font-bold"),
        "h3" => Some("text-xl font-bold"),
        "h4" => Some("text-lg font-bold"),
        "h5" => Some("text-base font-bold"),
        "h6" => Some("text-sm font-bold"),
        "button" => Some("cursor-pointer"),
        "a" => Some("cursor-pointer"),
        "input" => Some("px-2 py-1"),
        "textarea" => Some("px-2 py-1"),
        "ul" => Some("flex flex-col"),
        "ol" => Some("flex flex-col"),
        "li" => Some("flex items-center"),
        "p" => Some("text-base"),
        "label" => Some("text-sm"),
        "form" => Some("flex flex-col gap-4"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- lookup_color ---

    #[test]
    fn color_table_contains_full_tailwind_palette() {
        // 验证完整 Tailwind 色板：22 色系 × 11 色阶（复用模块级常量，单一数据源）
        for family in COLOR_FAMILIES {
            for shade in COLOR_SHADES {
                let key = format!("{family}_{shade}");
                assert!(
                    lookup_color(&key).is_some(),
                    "缺少颜色: {key}（请在 lookup_color 中补充）"
                );
            }
        }
    }

    #[test]
    fn color_table_contains_black_and_white() {
        assert_eq!(lookup_color("black"), Some(0x000000));
        assert_eq!(lookup_color("white"), Some(0xffffff));
    }

    #[test]
    fn color_table_spot_check_values() {
        // 抽检几个关键颜色值，防止复制粘贴错误
        assert_eq!(lookup_color("red_500"), Some(0xef4444));
        assert_eq!(lookup_color("blue_500"), Some(0x3b82f6));
        assert_eq!(lookup_color("green_500"), Some(0x22c55e));
        assert_eq!(lookup_color("gray_500"), Some(0x6b7280));
        assert_eq!(lookup_color("amber_500"), Some(0xf59e0b));
        assert_eq!(lookup_color("purple_500"), Some(0xa855f7));
    }

    #[test]
    fn color_table_unknown_returns_none() {
        assert_eq!(lookup_color("not_a_color"), None);
        assert_eq!(lookup_color("red_999"), None);
        assert_eq!(lookup_color(""), None);
    }

    // --- lookup_attr_method ---

    #[test]
    fn attr_method_camel_case_events() {
        assert_eq!(lookup_attr_method("onClick"), Some("on_click"));
        assert_eq!(lookup_attr_method("onMouseDown"), Some("on_mouse_down"));
        assert_eq!(lookup_attr_method("onKeyDown"), Some("on_key_down"));
        assert_eq!(lookup_attr_method("onDragMove"), Some("on_drag_move"));
        assert_eq!(lookup_attr_method("onBoxedAction"), Some("on_boxed_action"));
    }

    #[test]
    fn attr_method_snake_case_events() {
        assert_eq!(lookup_attr_method("on_click"), Some("on_click"));
        assert_eq!(lookup_attr_method("on_mouse_down"), Some("on_mouse_down"));
        assert_eq!(lookup_attr_method("on_key_down"), Some("on_key_down"));
    }

    #[test]
    fn attr_method_dimension_aliases() {
        assert_eq!(lookup_attr_method("width"), Some("w"));
        assert_eq!(lookup_attr_method("height"), Some("h"));
        assert_eq!(lookup_attr_method("minWidth"), Some("min_w"));
        assert_eq!(lookup_attr_method("maxHeight"), Some("max_h"));
        assert_eq!(lookup_attr_method("trackFocus"), Some("track_focus"));
        assert_eq!(lookup_attr_method("fontSize"), Some("text_size"));
        assert_eq!(lookup_attr_method("flexGrow"), None);
        assert_eq!(lookup_attr_method("flexShrink"), None);
        assert_eq!(lookup_attr_method("zIndex"), None);
    }

    #[test]
    fn attr_method_keeps_directional_border_value_methods() {
        assert_eq!(lookup_attr_method("border_t"), None);
        assert_eq!(lookup_attr_method("border_b"), None);
        assert_eq!(lookup_attr_method("border_l"), None);
        assert_eq!(lookup_attr_method("border_r"), None);
    }

    #[test]
    fn attr_flag_method_maps_directional_border_width_flags() {
        assert_eq!(lookup_attr_flag_method("border_t"), Some("border_t_1"));
        assert_eq!(lookup_attr_flag_method("border_b"), Some("border_b_1"));
        assert_eq!(lookup_attr_flag_method("border_l"), Some("border_l_1"));
        assert_eq!(lookup_attr_flag_method("border_r"), Some("border_r_1"));
    }

    #[test]
    fn attr_flag_method_maps_gpui_flex_flags() {
        assert_eq!(lookup_attr_flag_method("flexGrow"), Some("flex_grow"));
        assert_eq!(lookup_attr_flag_method("flexShrink"), Some("flex_shrink"));
    }

    #[test]
    fn attr_method_unknown_returns_none() {
        assert_eq!(lookup_attr_method("onAuxClick"), None);
        assert_eq!(lookup_attr_method("onFocus"), None);
        assert_eq!(lookup_attr_method("onMousePressure"), None);
        assert_eq!(lookup_attr_method("unknown_attr"), None);
        assert_eq!(lookup_attr_method(""), None);
    }

    // --- is_stateful_attr ---

    #[test]
    fn stateful_attr_detects_event_handlers() {
        assert!(is_stateful_attr("onClick"));
        assert!(is_stateful_attr("on_click"));
        assert!(is_stateful_attr("onDrag"));
        assert!(is_stateful_attr("on_hover"));
    }

    #[test]
    fn stateful_attr_detects_interactive_attrs() {
        // StatefulInteractiveElement 方法需要 ID。
        assert!(is_stateful_attr("active"));
        assert!(is_stateful_attr("focusable"));
        assert!(is_stateful_attr("overflow_scroll"));
        assert!(is_stateful_attr("tooltip"));
        assert!(is_stateful_attr("track_scroll"));
    }

    #[test]
    fn stateful_attr_ignores_styled_trait_methods() {
        // hover / focus / group 是 Styled trait 的样式方法，不需要 ID
        assert!(!is_stateful_attr("hover"));
        assert!(!is_stateful_attr("focus"));
        assert!(!is_stateful_attr("group"));
    }

    #[test]
    fn stateful_attr_ignores_non_stateful() {
        assert!(!is_stateful_attr("onMouseDown"));
        assert!(!is_stateful_attr("captureKeyDown"));
        assert!(!is_stateful_attr("class"));
        assert!(!is_stateful_attr("id"));
        assert!(!is_stateful_attr("flex"));
        assert!(!is_stateful_attr("gap"));
    }

    // --- lookup_spacing_method ---

    #[test]
    fn spacing_method_covers_all_directions() {
        assert_eq!(lookup_spacing_method("gap_"), Some("gap"));
        assert_eq!(lookup_spacing_method("gap_x_"), Some("gap_x"));
        assert_eq!(lookup_spacing_method("gap_y_"), Some("gap_y"));
        assert_eq!(lookup_spacing_method("p_"), Some("p"));
        assert_eq!(lookup_spacing_method("px_"), Some("px"));
        assert_eq!(lookup_spacing_method("py_"), Some("py"));
        assert_eq!(lookup_spacing_method("m_"), Some("m"));
        assert_eq!(lookup_spacing_method("mx_"), Some("mx"));
        assert_eq!(lookup_spacing_method("my_"), Some("my"));
        assert_eq!(lookup_spacing_method("w_"), Some("w"));
        assert_eq!(lookup_spacing_method("h_"), Some("h"));
        assert_eq!(lookup_spacing_method("min_w_"), Some("min_w"));
        assert_eq!(lookup_spacing_method("max_h_"), Some("max_h"));
    }

    #[test]
    fn spacing_method_unknown_returns_none() {
        assert_eq!(lookup_spacing_method("text_"), None);
        assert_eq!(lookup_spacing_method(""), None);
    }

    // --- is_valid_text_size ---

    #[test]
    fn text_size_validates_known_sizes() {
        for size in ["xs", "sm", "base", "lg", "xl", "2xl", "3xl"] {
            assert!(is_valid_text_size(size), "应接受文本大小: {size}");
        }
    }

    #[test]
    fn text_size_rejects_unknown() {
        assert!(!is_valid_text_size("huge"));
        assert!(!is_valid_text_size("4xl"));
        assert!(!is_valid_text_size("5xl"));
        assert!(!is_valid_text_size("6xl"));
        assert!(!is_valid_text_size(""));
    }
}
