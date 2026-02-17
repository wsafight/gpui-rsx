//! 静态查找表和常量定义
//!
//! 本模块包含所有编译期常量表，不依赖其他 codegen 子模块。

/// 事件处理器映射表：(camelCase, snake_case, method_name)
pub(crate) const EVENT_HANDLERS: &[(&str, &str, &str)] = &[
    ("onClick", "on_click", "on_click"),
    ("onMouseDown", "on_mouse_down", "on_mouse_down"),
    ("onMouseUp", "on_mouse_up", "on_mouse_up"),
    ("onMouseMove", "on_mouse_move", "on_mouse_move"),
    ("onMouseDownOut", "on_mouse_down_out", "on_mouse_down_out"),
    ("onMouseUpOut", "on_mouse_up_out", "on_mouse_up_out"),
    ("onKeyDown", "on_key_down", "on_key_down"),
    ("onKeyUp", "on_key_up", "on_key_up"),
    ("onFocus", "on_focus", "on_focus"),
    ("onBlur", "on_blur", "on_blur"),
    ("onHover", "on_hover", "on_hover"),
    ("onScrollWheel", "on_scroll_wheel", "on_scroll_wheel"),
    ("onDrag", "on_drag", "on_drag"),
    ("onDrop", "on_drop", "on_drop"),
    ("onAction", "on_action", "on_action"),
];

/// 属性名称映射表：(camelCase, snake_case)
/// 用于将 JSX 风格的 camelCase 属性名转换为 Rust 的 snake_case 方法名
pub(crate) const ATTRIBUTE_NAME_MAP: &[(&str, &str)] = &[
    // 层级和透明度
    ("zIndex", "z_index"),
    ("opacity", "opacity"),
    // 可见性
    ("visible", "visible"),
    // 定位
    ("top", "top"),
    ("left", "left"),
    ("right", "right"),
    ("bottom", "bottom"),
    ("inset", "inset"),
    // 尺寸
    ("width", "w"),
    ("height", "h"),
    ("minWidth", "min_w"),
    ("minHeight", "min_h"),
    ("maxWidth", "max_w"),
    ("maxHeight", "max_h"),
    // 间距方向
    ("gapX", "gap_x"),
    ("gapY", "gap_y"),
    // Flex
    ("flexBasis", "basis"),
    ("flexGrow", "flex_grow"),
    ("flexShrink", "flex_shrink"),
    ("flexOrder", "order"),
    // 文本
    ("fontSize", "font_size"),
    ("lineHeight", "line_height"),
    ("fontWeight", "font_weight"),
    ("textAlign", "text_align"),
    ("textDecoration", "text_decoration"),
    // 边框
    ("borderRadius", "border_radius"),
    ("borderTop", "border_t"),
    ("borderBottom", "border_b"),
    ("borderLeft", "border_l"),
    ("borderRight", "border_r"),
    // 方向性圆角
    ("roundedTop", "rounded_t"),
    ("roundedBottom", "rounded_b"),
    ("roundedTopLeft", "rounded_tl"),
    ("roundedTopRight", "rounded_tr"),
    ("roundedBottomLeft", "rounded_bl"),
    ("roundedBottomRight", "rounded_br"),
    // 阴影
    ("boxShadow", "shadow"),
    // 溢出
    ("overflowX", "overflow_x_hidden"),
    ("overflowY", "overflow_y_hidden"),
];

/// 颜色映射表 — 完整 Tailwind 色板
pub(crate) const COLOR_MAP: &[(&str, u32)] = &[
    // Slate
    ("slate_50", 0xf8fafc),
    ("slate_100", 0xf1f5f9),
    ("slate_200", 0xe2e8f0),
    ("slate_300", 0xcbd5e1),
    ("slate_400", 0x94a3b8),
    ("slate_500", 0x64748b),
    ("slate_600", 0x475569),
    ("slate_700", 0x334155),
    ("slate_800", 0x1e293b),
    ("slate_900", 0x0f172a),
    ("slate_950", 0x020617),
    // Gray
    ("gray_50", 0xf9fafb),
    ("gray_100", 0xf3f4f6),
    ("gray_200", 0xe5e7eb),
    ("gray_300", 0xd1d5db),
    ("gray_400", 0x9ca3af),
    ("gray_500", 0x6b7280),
    ("gray_600", 0x4b5563),
    ("gray_700", 0x374151),
    ("gray_800", 0x1f2937),
    ("gray_900", 0x111827),
    ("gray_950", 0x030712),
    // Zinc
    ("zinc_50", 0xfafafa),
    ("zinc_100", 0xf4f4f5),
    ("zinc_200", 0xe4e4e7),
    ("zinc_300", 0xd4d4d8),
    ("zinc_400", 0xa1a1aa),
    ("zinc_500", 0x71717a),
    ("zinc_600", 0x52525b),
    ("zinc_700", 0x3f3f46),
    ("zinc_800", 0x27272a),
    ("zinc_900", 0x18181b),
    ("zinc_950", 0x09090b),
    // Neutral
    ("neutral_50", 0xfafafa),
    ("neutral_100", 0xf5f5f5),
    ("neutral_200", 0xe5e5e5),
    ("neutral_300", 0xd4d4d4),
    ("neutral_400", 0xa3a3a3),
    ("neutral_500", 0x737373),
    ("neutral_600", 0x525252),
    ("neutral_700", 0x404040),
    ("neutral_800", 0x262626),
    ("neutral_900", 0x171717),
    ("neutral_950", 0x0a0a0a),
    // Stone
    ("stone_50", 0xfafaf9),
    ("stone_100", 0xf5f5f4),
    ("stone_200", 0xe7e5e4),
    ("stone_300", 0xd6d3d1),
    ("stone_400", 0xa8a29e),
    ("stone_500", 0x78716c),
    ("stone_600", 0x57534e),
    ("stone_700", 0x44403c),
    ("stone_800", 0x292524),
    ("stone_900", 0x1c1917),
    ("stone_950", 0x0c0a09),
    // Red
    ("red_50", 0xfef2f2),
    ("red_100", 0xfee2e2),
    ("red_200", 0xfecaca),
    ("red_300", 0xfca5a5),
    ("red_400", 0xf87171),
    ("red_500", 0xef4444),
    ("red_600", 0xdc2626),
    ("red_700", 0xb91c1c),
    ("red_800", 0x991b1b),
    ("red_900", 0x7f1d1d),
    ("red_950", 0x450a0a),
    // Orange
    ("orange_50", 0xfff7ed),
    ("orange_100", 0xffedd5),
    ("orange_200", 0xfed7aa),
    ("orange_300", 0xfdba74),
    ("orange_400", 0xfb923c),
    ("orange_500", 0xf97316),
    ("orange_600", 0xea580c),
    ("orange_700", 0xc2410c),
    ("orange_800", 0x9a3412),
    ("orange_900", 0x7c2d12),
    ("orange_950", 0x431407),
    // Amber
    ("amber_50", 0xfffbeb),
    ("amber_100", 0xfef3c7),
    ("amber_200", 0xfde68a),
    ("amber_300", 0xfcd34d),
    ("amber_400", 0xfbbf24),
    ("amber_500", 0xf59e0b),
    ("amber_600", 0xd97706),
    ("amber_700", 0xb45309),
    ("amber_800", 0x92400e),
    ("amber_900", 0x78350f),
    ("amber_950", 0x451a03),
    // Yellow
    ("yellow_50", 0xfefce8),
    ("yellow_100", 0xfef9c3),
    ("yellow_200", 0xfef08a),
    ("yellow_300", 0xfde047),
    ("yellow_400", 0xfacc15),
    ("yellow_500", 0xeab308),
    ("yellow_600", 0xca8a04),
    ("yellow_700", 0xa16207),
    ("yellow_800", 0x854d0e),
    ("yellow_900", 0x713f12),
    ("yellow_950", 0x422006),
    // Lime
    ("lime_50", 0xf7fee7),
    ("lime_100", 0xecfccb),
    ("lime_200", 0xd9f99d),
    ("lime_300", 0xbef264),
    ("lime_400", 0xa3e635),
    ("lime_500", 0x84cc16),
    ("lime_600", 0x65a30d),
    ("lime_700", 0x4d7c0f),
    ("lime_800", 0x3f6212),
    ("lime_900", 0x365314),
    ("lime_950", 0x1a2e05),
    // Green
    ("green_50", 0xf0fdf4),
    ("green_100", 0xdcfce7),
    ("green_200", 0xbbf7d0),
    ("green_300", 0x86efac),
    ("green_400", 0x4ade80),
    ("green_500", 0x22c55e),
    ("green_600", 0x16a34a),
    ("green_700", 0x15803d),
    ("green_800", 0x166534),
    ("green_900", 0x14532d),
    ("green_950", 0x052e16),
    // Emerald
    ("emerald_50", 0xecfdf5),
    ("emerald_100", 0xd1fae5),
    ("emerald_200", 0xa7f3d0),
    ("emerald_300", 0x6ee7b7),
    ("emerald_400", 0x34d399),
    ("emerald_500", 0x10b981),
    ("emerald_600", 0x059669),
    ("emerald_700", 0x047857),
    ("emerald_800", 0x065f46),
    ("emerald_900", 0x064e3b),
    ("emerald_950", 0x022c22),
    // Teal
    ("teal_50", 0xf0fdfa),
    ("teal_100", 0xccfbf1),
    ("teal_200", 0x99f6e4),
    ("teal_300", 0x5eead4),
    ("teal_400", 0x2dd4bf),
    ("teal_500", 0x14b8a6),
    ("teal_600", 0x0d9488),
    ("teal_700", 0x0f766e),
    ("teal_800", 0x115e59),
    ("teal_900", 0x134e4a),
    ("teal_950", 0x042f2e),
    // Cyan
    ("cyan_50", 0xecfeff),
    ("cyan_100", 0xcffafe),
    ("cyan_200", 0xa5f3fc),
    ("cyan_300", 0x67e8f9),
    ("cyan_400", 0x22d3ee),
    ("cyan_500", 0x06b6d4),
    ("cyan_600", 0x0891b2),
    ("cyan_700", 0x0e7490),
    ("cyan_800", 0x155e75),
    ("cyan_900", 0x164e63),
    ("cyan_950", 0x083344),
    // Sky
    ("sky_50", 0xf0f9ff),
    ("sky_100", 0xe0f2fe),
    ("sky_200", 0xbae6fd),
    ("sky_300", 0x7dd3fc),
    ("sky_400", 0x38bdf8),
    ("sky_500", 0x0ea5e9),
    ("sky_600", 0x0284c7),
    ("sky_700", 0x0369a1),
    ("sky_800", 0x075985),
    ("sky_900", 0x0c4a6e),
    ("sky_950", 0x082f49),
    // Blue
    ("blue_50", 0xeff6ff),
    ("blue_100", 0xdbeafe),
    ("blue_200", 0xbfdbfe),
    ("blue_300", 0x93c5fd),
    ("blue_400", 0x60a5fa),
    ("blue_500", 0x3b82f6),
    ("blue_600", 0x2563eb),
    ("blue_700", 0x1d4ed8),
    ("blue_800", 0x1e40af),
    ("blue_900", 0x1e3a8a),
    ("blue_950", 0x172554),
    // Indigo
    ("indigo_50", 0xeef2ff),
    ("indigo_100", 0xe0e7ff),
    ("indigo_200", 0xc7d2fe),
    ("indigo_300", 0xa5b4fc),
    ("indigo_400", 0x818cf8),
    ("indigo_500", 0x6366f1),
    ("indigo_600", 0x4f46e5),
    ("indigo_700", 0x4338ca),
    ("indigo_800", 0x3730a3),
    ("indigo_900", 0x312e81),
    ("indigo_950", 0x1e1b4b),
    // Violet
    ("violet_50", 0xf5f3ff),
    ("violet_100", 0xede9fe),
    ("violet_200", 0xddd6fe),
    ("violet_300", 0xc4b5fd),
    ("violet_400", 0xa78bfa),
    ("violet_500", 0x8b5cf6),
    ("violet_600", 0x7c3aed),
    ("violet_700", 0x6d28d9),
    ("violet_800", 0x5b21b6),
    ("violet_900", 0x4c1d95),
    ("violet_950", 0x2e1065),
    // Purple
    ("purple_50", 0xfaf5ff),
    ("purple_100", 0xf3e8ff),
    ("purple_200", 0xe9d5ff),
    ("purple_300", 0xd8b4fe),
    ("purple_400", 0xc084fc),
    ("purple_500", 0xa855f7),
    ("purple_600", 0x9333ea),
    ("purple_700", 0x7e22ce),
    ("purple_800", 0x6b21a8),
    ("purple_900", 0x581c87),
    ("purple_950", 0x3b0764),
    // Fuchsia
    ("fuchsia_50", 0xfdf4ff),
    ("fuchsia_100", 0xfae8ff),
    ("fuchsia_200", 0xf5d0fe),
    ("fuchsia_300", 0xf0abfc),
    ("fuchsia_400", 0xe879f9),
    ("fuchsia_500", 0xd946ef),
    ("fuchsia_600", 0xc026d3),
    ("fuchsia_700", 0xa21caf),
    ("fuchsia_800", 0x86198f),
    ("fuchsia_900", 0x701a75),
    ("fuchsia_950", 0x4a044e),
    // Pink
    ("pink_50", 0xfdf2f8),
    ("pink_100", 0xfce7f3),
    ("pink_200", 0xfbcfe8),
    ("pink_300", 0xf9a8d4),
    ("pink_400", 0xf472b6),
    ("pink_500", 0xec4899),
    ("pink_600", 0xdb2777),
    ("pink_700", 0xbe185d),
    ("pink_800", 0x9d174d),
    ("pink_900", 0x831843),
    ("pink_950", 0x500724),
    // Rose
    ("rose_50", 0xfff1f2),
    ("rose_100", 0xffe4e6),
    ("rose_200", 0xfecdd3),
    ("rose_300", 0xfda4af),
    ("rose_400", 0xfb7185),
    ("rose_500", 0xf43f5e),
    ("rose_600", 0xe11d48),
    ("rose_700", 0xbe123c),
    ("rose_800", 0x9f1239),
    ("rose_900", 0x881337),
    ("rose_950", 0x4c0519),
    // 特殊颜色
    ("white", 0xffffff),
    ("black", 0x000000),
];

/// 标签默认样式表：(标签名, class 字符串)
/// 仅当元素带 `styled` 标志时应用，复用 parse_class_string() 解析
pub(crate) const TAG_DEFAULT_STYLES: &[(&str, &str)] = &[
    // 标题
    ("h1", "text-3xl font-bold"),
    ("h2", "text-2xl font-bold"),
    ("h3", "text-xl font-bold"),
    ("h4", "text-lg font-bold"),
    ("h5", "text-base font-bold"),
    ("h6", "text-sm font-bold"),
    // 交互
    ("button", "cursor-pointer"),
    ("a", "cursor-pointer"),
    // 表单
    ("input", "px-2 py-1"),
    ("textarea", "px-2 py-1"),
    // 列表
    ("ul", "flex flex-col"),
    ("ol", "flex flex-col"),
];

/// 间距/尺寸 class 前缀映射表
pub(crate) const SPACING_PATTERNS: &[(&str, &str)] = &[
    ("gap_", "gap"),
    ("p_", "p"),
    ("px_", "px"),
    ("py_", "py"),
    ("pt_", "pt"),
    ("pb_", "pb"),
    ("pl_", "pl"),
    ("pr_", "pr"),
    ("m_", "m"),
    ("mx_", "mx"),
    ("my_", "my"),
    ("mt_", "mt"),
    ("mb_", "mb"),
    ("ml_", "ml"),
    ("mr_", "mr"),
    ("w_", "w"),
    ("h_", "h"),
];

/// 有效的 text 大小名称白名单
pub(crate) const VALID_TEXT_SIZES: &[&str] =
    &["xs", "sm", "base", "lg", "xl", "2xl", "3xl", "4xl", "5xl"];

/// 查找颜色值的辅助函数
///
/// 在颜色映射表中查找给定名称的颜色值。
///
/// # 参数
/// - `name`: 颜色名称（如 "red_500", "blue_600"）
///
/// # 返回值
/// - `Some(u32)`: 找到颜色，返回 RGB 十六进制值
/// - `None`: 未找到颜色
pub(crate) fn lookup_color(name: &str) -> Option<u32> {
    COLOR_MAP
        .iter()
        .find(|(color_name, _)| *color_name == name)
        .map(|(_, color_value)| *color_value)
}
