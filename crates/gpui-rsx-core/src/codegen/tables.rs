//! 静态查找表：手写“语义层”
//!
//! GPUI 本身的 API 事实（方法名、参数个数、所属 trait、camelCase 别名、Tailwind 色板）
//! 由 `cargo xtask gen-tables` 从 GPUI 源码生成到 `tables_generated.rs`，这里不再手工同步。
//!
//! 本模块只保留 GPUI 中没有直接对应、需要人为决定的部分：
//! - RSX 属性别名（`width` → `w`、`fontSize` → `text_size` 等非机械映射）
//! - class 别名（`border` → `border_1`、`debug-outline` 等）
//! - Tailwind 前缀到长度方法的映射、标签默认样式
//! - 动态 class 快速路径的取舍策略

use std::sync::OnceLock;

pub use super::tables_generated::{
    COLOR_FAMILIES, COLOR_SHADES, GPUI_COMPONENT_REV, GPUI_COMPONENT_THEME_COLORS, GPUI_METHODS,
    GPUI_SOURCE_REV, GpuiMethod, T_FLUENT, T_INTERACTIVE, T_PARENT, T_STATEFUL, T_STYLED,
    lookup_color, lookup_gpui_camel_alias, lookup_gpui_method,
};

/// 查找属性的 GPUI 方法名（事件处理器 + camelCase 属性映射）。
#[derive(Clone, Copy)]
pub struct AttrMethodInfo {
    pub method: &'static str,
    pub needs_id: bool,
    pub multi_arg: bool,
}

/// 语义别名：camelCase 机械转换无法得到的 RSX 属性名。
fn lookup_semantic_attr_alias(name: &str) -> Option<&'static str> {
    match name {
        "width" => Some("w"),
        "height" => Some("h"),
        "minWidth" => Some("min_w"),
        "minHeight" => Some("min_h"),
        "maxWidth" => Some("max_w"),
        "maxHeight" => Some("max_h"),
        "fontSize" => Some("text_size"),
        "backgroundColor" => Some("bg"),
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
        "ariaKeyShortcuts" => Some("aria_keyshortcuts"),
        // Inherent `Div` / `Img` methods that are not part of the extracted traits.
        "imageCache" => Some("image_cache"),
        "objectFit" => Some("object_fit"),
        "onChildrenPrepainted" => Some("on_children_prepainted"),
        "withFallback" => Some("with_fallback"),
        "withLoading" => Some("with_loading"),
        "gridCols" => Some("grid_cols"),
        "gridRows" => Some("grid_rows"),
        "gridColsMinContent" => Some("grid_cols_min_content"),
        "gridColsMaxContent" => Some("grid_cols_max_content"),
        "gridRowsMinContent" => Some("grid_rows_min_content"),
        "gridRowsMaxContent" => Some("grid_rows_max_content"),
        "colSpan" => Some("col_span"),
        "rowSpan" => Some("row_span"),
        "colStart" => Some("col_start"),
        "colEnd" => Some("col_end"),
        "rowStart" => Some("row_start"),
        "rowEnd" => Some("row_end"),
        "scrollbarWidth" => Some("scrollbar_width"),
        _ => None,
    }
}

/// RSX 属性名 → GPUI 方法名。
///
/// 优先级：语义别名 → 生成的 camelCase 别名 → 与 GPUI 方法同名（snake_case 原样使用）。
pub fn lookup_attr_method(name: &str) -> Option<&'static str> {
    lookup_semantic_attr_alias(name)
        .or_else(|| lookup_gpui_camel_alias(name))
        .or_else(|| lookup_gpui_method(name).map(|m| m.name))
}

pub fn lookup_attr_method_info(name: &str) -> Option<AttrMethodInfo> {
    let method = lookup_attr_method(name)?;
    let (needs_id, multi_arg) = method_properties(method);
    Some(AttrMethodInfo {
        method,
        needs_id,
        multi_arg,
    })
}

/// 查找 flag 属性的 GPUI 方法名。
///
/// 绝大多数 flag 属性可直接复用 value 属性映射；方向性 border 是例外：
/// `border_t={value}` 应调用 `.border_t(value)`，而 `border_t` flag 应调用
/// GPUI 的预设宽度方法 `.border_t_1()`。
pub fn lookup_attr_flag_method(name: &str) -> Option<&'static str> {
    match name {
        "flexGrow" => Some("flex_grow_1"),
        "flexShrink" => Some("flex_shrink_1"),
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
/// 只有 `StatefulInteractiveElement` 上的方法需要先调用 `.id()`；该集合由生成器维护。
pub fn is_stateful_attr(name: &str) -> bool {
    if let Some(info) = lookup_attr_method_info(name) {
        return info.needs_id;
    }

    is_stateful_method(name)
}

pub fn is_stateful_method(method: &str) -> bool {
    method_properties(method).0
}

/// `(needs_id, multi_arg)`，直接来自生成的 GPUI 方法事实。
fn method_properties(method: &str) -> (bool, bool) {
    match lookup_gpui_method(method) {
        Some(m) => (m.traits & T_STATEFUL != 0, m.arity >= 2),
        None => (false, false),
    }
}

/// Tailwind 状态前缀（`hover:`、`active:` 等）中需要 stateful element 的那些。
pub fn is_stateful_variant(variant: &str) -> bool {
    variant == "active" || variant == "group-active" || variant.starts_with("group-active/")
}

/// 检查静态 class 是否会调用 `StatefulInteractiveElement` 方法。
pub fn is_stateful_class(class: &str) -> bool {
    if let Some((variant, _)) = class.split_once(':') {
        return is_stateful_variant(variant);
    }
    gpui_no_arg_class_method(class).is_some_and(|m| m.traits & T_STATEFUL != 0)
}

/// 查找间距/尺寸 class 前缀（下划线形式，如 `"gap_x_"`）对应的 GPUI 方法名。
pub fn lookup_spacing_method(prefix: &str) -> Option<&'static str> {
    let method = prefix.strip_suffix('_')?;
    LENGTH_CLASS_SPECS
        .iter()
        .find(|spec| spec.method == method)
        .map(|spec| spec.method)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LengthClassFamily {
    Sizing,
    Spacing,
}

impl LengthClassFamily {
    pub const fn allows_percent(self) -> bool {
        matches!(self, Self::Sizing)
    }

    pub const fn allows_fraction(self) -> bool {
        matches!(self, Self::Sizing)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LengthClassSpec {
    pub prefix: &'static str,
    pub method: &'static str,
    pub family: LengthClassFamily,
}

impl LengthClassSpec {
    const fn sizing(prefix: &'static str, method: &'static str) -> Self {
        Self {
            prefix,
            method,
            family: LengthClassFamily::Sizing,
        }
    }

    const fn spacing(prefix: &'static str, method: &'static str) -> Self {
        Self {
            prefix,
            method,
            family: LengthClassFamily::Spacing,
        }
    }
}

/// Shared length-family table for static and dynamic class parsing.
///
/// Order matters: longer prefixes such as `min-w-` and `gap-x-` must be checked before
/// shorter prefixes like `w-` and `gap-`. Every `method` is checked against the generated
/// GPUI facts by a unit test.
pub const LENGTH_CLASS_SPECS: &[LengthClassSpec] = &[
    LengthClassSpec::spacing("rounded-", "rounded"),
    LengthClassSpec::spacing("leading-", "line_height"),
    LengthClassSpec::sizing("min-w-", "min_w"),
    LengthClassSpec::sizing("max-w-", "max_w"),
    LengthClassSpec::sizing("min-h-", "min_h"),
    LengthClassSpec::sizing("max-h-", "max_h"),
    LengthClassSpec::spacing("gap-x-", "gap_x"),
    LengthClassSpec::spacing("gap-y-", "gap_y"),
    LengthClassSpec::sizing("size-", "size"),
    LengthClassSpec::spacing("gap-", "gap"),
    LengthClassSpec::spacing("px-", "px"),
    LengthClassSpec::spacing("py-", "py"),
    LengthClassSpec::spacing("pt-", "pt"),
    LengthClassSpec::spacing("pb-", "pb"),
    LengthClassSpec::spacing("pl-", "pl"),
    LengthClassSpec::spacing("pr-", "pr"),
    LengthClassSpec::spacing("mx-", "mx"),
    LengthClassSpec::spacing("my-", "my"),
    LengthClassSpec::spacing("mt-", "mt"),
    LengthClassSpec::spacing("mb-", "mb"),
    LengthClassSpec::spacing("ml-", "ml"),
    LengthClassSpec::spacing("mr-", "mr"),
    LengthClassSpec::sizing("w-", "w"),
    LengthClassSpec::sizing("h-", "h"),
    LengthClassSpec::spacing("p-", "p"),
    LengthClassSpec::spacing("m-", "m"),
];

pub fn split_length_class(class: &str) -> Option<(&'static str, &str, LengthClassFamily)> {
    for spec in LENGTH_CLASS_SPECS {
        if let Some(value) = class.strip_prefix(spec.prefix) {
            return Some((spec.method, value, spec.family));
        }
    }

    None
}

/// class 别名：语义上属于 Tailwind、但 GPUI 方法名不同或需要特殊展开的无参 class。
pub const SEMANTIC_NO_ARG_CLASSES: &[&str] = &[
    "border",
    "debug-outline",
    "flex-grow",
    "flex-shrink",
    "group",
    "no-underline",
];

/// 永远不能作为 class 使用的 GPUI 无参方法（返回引用或仅在 debug 构建存在）。
const NON_CLASS_METHODS: &[&str] = &[
    "debug",
    "debug_below",
    "interactivity",
    "style",
    "text_style",
];

/// `lookup_gpui_method(&class.replace('-', "_"))` without allocating; this runs for every class.
fn lookup_gpui_method_kebab(class: &str) -> Option<&'static GpuiMethod> {
    let snake = || {
        class
            .bytes()
            .map(|byte| if byte == b'-' { b'_' } else { byte })
    };
    GPUI_METHODS
        .binary_search_by(|method| method.name.bytes().cmp(snake()))
        .ok()
        .map(|index| &GPUI_METHODS[index])
}

/// class 名是否对应一个可以直接调用的 GPUI 无参方法。
pub fn gpui_no_arg_class_method(class: &str) -> Option<&'static GpuiMethod> {
    if class.is_empty()
        || !class
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return None;
    }
    let method = lookup_gpui_method_kebab(class)?;
    (method.arity == 0
        && method.traits & (T_STYLED | T_INTERACTIVE | T_STATEFUL) != 0
        && !NON_CLASS_METHODS.contains(&method.name))
    .then_some(method)
}

/// 严格模式下“无参 class”是否受支持：语义别名或任意 GPUI 无参样式方法。
pub fn is_supported_common_no_arg_class(class: &str) -> bool {
    SEMANTIC_NO_ARG_CLASSES.contains(&class) || gpui_no_arg_class_method(class).is_some()
}

/// 动态 class 快速路径额外收录的语义 class（含常用数值，避免运行时数字解析）。
const DYNAMIC_SEMANTIC_CLASSES: &[&str] = &[
    "border",
    "border-t",
    "border-b",
    "border-l",
    "border-r",
    "border-x",
    "border-y",
    "debug-outline",
    "flex-grow",
    "flex-shrink",
    "no-underline",
    "font-thin",
    "font-extralight",
    "font-light",
    "font-normal",
    "font-medium",
    "font-semibold",
    "font-bold",
    "font-extrabold",
    "font-black",
    "opacity-0",
    "opacity-25",
    "opacity-50",
    "opacity-75",
    "opacity-100",
    "gap-1",
    "gap-2",
    "gap-3",
    "gap-4",
    "gap-6",
    "gap-8",
    "p-1",
    "p-2",
    "p-3",
    "p-4",
    "p-6",
    "p-8",
    "px-2",
    "px-3",
    "px-4",
    "py-1",
    "py-2",
    "py-3",
    "py-4",
    "m-1",
    "m-2",
    "m-4",
    "mt-2",
    "mt-4",
    "mb-2",
    "mb-4",
];

/// 动态 class 快速路径是否收录某个生成的 GPUI 方法。
///
/// 盒模型家族（`p_4`、`w_1_2`、`m_neg_4` …）有上千个预设，动态路径用数值回退解析处理，
/// 不逐个展开；只保留非数值的 `-full` / `-auto` / `-px` 预设。
fn dynamic_includes(method: &GpuiMethod) -> bool {
    if method.arity != 0
        || method.traits & T_STYLED == 0
        || method.traits & T_STATEFUL != 0
        || NON_CLASS_METHODS.contains(&method.name)
    {
        return false;
    }
    match (method.family_prefix, method.family_suffix) {
        ("", "") => true,
        (prefix, _) if prefix.starts_with("rounded") || prefix.starts_with("border") => true,
        ("w" | "h" | "size" | "min_w" | "max_w" | "min_h" | "max_h", "full" | "auto" | "px") => {
            true
        }
        ("m" | "mx" | "my" | "mt" | "mb" | "ml" | "mr", "auto") => !method.negated,
        _ => false,
    }
}

/// 动态 class 快速路径（运行时 match 表）收录的 class 名。
pub fn dynamic_common_classes() -> impl Iterator<Item = &'static str> {
    static CLASSES: OnceLock<Vec<&'static str>> = OnceLock::new();
    CLASSES
        .get_or_init(|| {
            let mut classes: Vec<&'static str> = GPUI_METHODS
                .iter()
                .filter(|m| dynamic_includes(m))
                .map(|m| &*Box::leak(m.name.replace('_', "-").into_boxed_str()))
                .collect();
            for class in DYNAMIC_SEMANTIC_CLASSES {
                if !classes.contains(class) {
                    classes.push(class);
                }
            }
            classes
        })
        .iter()
        .copied()
}

/// 检查是否是有效的文本大小名称（`xs`…`xl`、`2xl`…），且 GPUI 存在对应的 `text_*` 方法。
pub fn is_valid_text_size(size: &str) -> bool {
    let shaped = matches!(size, "xs" | "sm" | "base" | "lg" | "xl")
        || size
            .strip_suffix("xl")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
    shaped
        && matches!(
            lookup_gpui_method(&format!("text_{size}")),
            Some(m) if m.arity == 0
        )
}

/// 查找标签默认样式（仅当元素带 `styled` 标志时使用）
pub fn lookup_tag_default(tag: &str) -> Option<&'static str> {
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

/// HTML-like tags that expand to `div()`.
pub const HTML_TAGS: &[&str] = &[
    "div", "span", "section", "article", "header", "footer", "main", "nav", "aside", "h1", "h2",
    "h3", "h4", "h5", "h6", "p", "label", "a", "button", "input", "textarea", "select", "form",
    "ul", "ol", "li",
];

pub const FONT_WEIGHTS: &[(&str, &str)] = &[
    ("font-thin", "THIN"),
    ("font-extralight", "EXTRA_LIGHT"),
    ("font-light", "LIGHT"),
    ("font-normal", "NORMAL"),
    ("font-medium", "MEDIUM"),
    ("font-semibold", "SEMIBOLD"),
    ("font-bold", "BOLD"),
    ("font-extrabold", "EXTRA_BOLD"),
    ("font-black", "BLACK"),
];

#[cfg(test)]
mod tests;
