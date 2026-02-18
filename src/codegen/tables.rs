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
        "onKeyDown" | "on_key_down" => Some("on_key_down"),
        "onKeyUp" | "on_key_up" => Some("on_key_up"),
        "onFocus" | "on_focus" => Some("on_focus"),
        "onBlur" | "on_blur" => Some("on_blur"),
        "onHover" | "on_hover" => Some("on_hover"),
        "onScrollWheel" | "on_scroll_wheel" => Some("on_scroll_wheel"),
        "onDrag" | "on_drag" => Some("on_drag"),
        "onDrop" | "on_drop" => Some("on_drop"),
        "onAction" | "on_action" => Some("on_action"),
        // 属性名称映射（camelCase → snake_case，仅非恒等映射）
        "zIndex" => Some("z_index"),
        "width" => Some("w"),
        "height" => Some("h"),
        "minWidth" => Some("min_w"),
        "minHeight" => Some("min_h"),
        "maxWidth" => Some("max_w"),
        "maxHeight" => Some("max_h"),
        "gapX" => Some("gap_x"),
        "gapY" => Some("gap_y"),
        "flexBasis" => Some("basis"),
        "flexGrow" => Some("flex_grow"),
        "flexShrink" => Some("flex_shrink"),
        "flexOrder" => Some("order"),
        "fontSize" => Some("font_size"),
        "lineHeight" => Some("line_height"),
        "fontWeight" => Some("font_weight"),
        "textAlign" => Some("text_align"),
        "textDecoration" => Some("text_decoration"),
        "borderRadius" => Some("border_radius"),
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
        "overflowX" => Some("overflow_x"),
        "overflowY" => Some("overflow_y"),
        _ => None,
    }
}

/// 检查属性是否需要 stateful element（需要 `.id()`）
///
/// 事件处理器和部分交互属性需要 `StatefulInteractiveElement` trait，
/// 该 trait 要求元素先调用 `.id()`。
pub(crate) fn is_stateful_attr(name: &str) -> bool {
    // 事件处理器：snake_case 以 "on_" 开头，或 camelCase 以 "on" 开头且第三字符为大写
    if name.starts_with("on_")
        || (name.starts_with("on")
            && name
                .as_bytes()
                .get(2)
                .is_some_and(|b| b.is_ascii_uppercase()))
    {
        return true;
    }
    // 其他 stateful 属性
    matches!(
        name,
        "hover" | "active" | "focus" | "tooltip" | "group" | "track_focus"
    )
}

/// 查找间距/尺寸 class 前缀对应的 GPUI 方法名
///
/// 使用 match 替代原 SPACING_PATTERNS 数组的线性扫描（17 项 → O(1)）。
pub(crate) fn lookup_spacing_method(prefix: &str) -> Option<&'static str> {
    match prefix {
        "gap_" => Some("gap"),
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
        _ => None,
    }
}

/// 检查是否是有效的文本大小名称
///
/// 使用 match 替代原 VALID_TEXT_SIZES 数组的 `.contains()` 线性扫描。
pub(crate) fn is_valid_text_size(size: &str) -> bool {
    matches!(
        size,
        "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl"
    )
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
        _ => None,
    }
}
