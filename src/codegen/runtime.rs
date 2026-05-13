//! 运行时 class 处理
//!
//! 生成运行时动态 class 字符串解析和应用的代码。
//! 当 class 属性的值是表达式而非字符串字面量时使用。
//!
//! 优化：使用 thread_local 缓存 match 表，避免多个动态 class 重复生成相同 TokenStream。

use super::class::{ClassMode, parse_single_class_with_mode};
use super::tables::{COLOR_FAMILIES, COLOR_SHADES, lookup_color};
use proc_macro2::TokenStream;
use quote::quote;
use std::cell::RefCell;

// 缓存所有 match 分支拼接后的字符串（thread_local 保证编译过程中只生成一次）
//
// 注意：不能缓存 proc_macro2::TokenStream，因为它的 token handle 绑定到
// 当前 proc macro 调用的 bridge 连接。每次调用结束后 bridge 失效，
// 下次调用时旧 handle 变成悬垂引用，导致 "use-after-free" panic。
//
// 优化：将所有 match arm 拼接为单个字符串，每次宏调用只做 1 次 parse，
// 而非对每个 arm 分别 parse（原先每次调用需数百次 parse）。
thread_local! {
    static COMMON_CLASS_MATCHES_STR: RefCell<Option<String>> = const { RefCell::new(None) };
    static COLOR_FALLBACK_STR: RefCell<Option<String>> = const { RefCell::new(None) };
    static NUMERIC_FALLBACK_STR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// 获取 common class match 表（惰性初始化字符串缓存，每次返回当前 bridge 的新 TokenStream）
fn get_cached_common_class_matches() -> TokenStream {
    COMMON_CLASS_MATCHES_STR.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let s = borrow.get_or_insert_with(|| {
            // 将所有 match arm 拼接为单个字符串，避免后续每次调用时逐条解析
            generate_common_class_matches()
                .into_iter()
                .map(|ts| ts.to_string())
                .collect::<String>()
        });
        s.parse::<TokenStream>()
            .expect("cached match arms are valid")
    })
}

/// 获取颜色回退代码（惰性初始化字符串缓存，每次返回当前 bridge 的新 TokenStream）
fn get_cached_color_fallback() -> TokenStream {
    COLOR_FALLBACK_STR.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let s = borrow.get_or_insert_with(|| generate_color_fallback_code().to_string());
        s.parse::<TokenStream>()
            .expect("cached color fallback is valid")
    })
}

/// 获取数值回退代码（惰性初始化字符串缓存，每次返回当前 bridge 的新 TokenStream）
fn get_cached_numeric_fallback() -> TokenStream {
    NUMERIC_FALLBACK_STR.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let s = borrow.get_or_insert_with(|| generate_numeric_fallback_code().to_string());
        s.parse::<TokenStream>()
            .expect("cached numeric fallback is valid")
    })
}

/// 生成运行时 class 解析代码
///
/// 当 class 属性是动态表达式时，生成一个在运行时解析和应用 class 的闭包。
///
/// # 支持的 class
///
/// 动态 class 支持两类解析：
/// 1. **静态 match 表**（快速路径）：[`generate_common_class_matches`] 中的预编译常用 class
/// 2. **颜色前缀解析**：完整 Tailwind 色板 + `[#rgb]` / `[#rrggbb]` arbitrary hex
/// 3. **数值前缀解析**（通用路径）：对间距/尺寸/透明度类，支持任意数值
///    - `gap-7`、`gap-x-3`、`p-5`、`px-7`、`m-3`、`ml-5`、`w-48`、`h-16` 等
///    - `opacity-33` 等
///    - 静态 match 未命中时自动回退到此路径，无需扩充预编译列表
/// 4. **其余 class**（如 Tailwind variants、自定义 class）静默忽略
///
/// 推荐方案（按性能从高到低）：
/// 1. **字符串字面量**（最佳）：`class="flex gap-4"` → 编译期展开，支持所有 class
/// 2. **条件表达式**（次佳）：`class={if active { "flex" } else { "block" }}`
/// 3. **动态表达式**：`class={dynamic_str}` → 支持间距/尺寸/透明度和 arbitrary hex 颜色
///
/// # 代码体积优化
///
/// match 表被提取到 `#[inline(never)]` 泛型局部函数中，带来两个好处：
/// 1. 同一元素类型的多个 `class={expr}` 共享同一份单态化实例（LLVM ICF 合并）
/// 2. `#[inline(never)]` 阻止 match 表被内联到父函数，减少指令缓存压力
///
/// # 生成的代码模式
///
/// ```ignore
/// {
///     #[inline(never)]
///     fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {
///         match class {
///             "flex" => el.flex(),
///             "gap-4" => el.gap(px(4.0)),
///             _ => {
///                 // 数值前缀回退：处理任意数值（gap-7、p-5 等）
///                 if let Some(rest) = class.strip_prefix("gap-") {
///                     if let Ok(n) = rest.parse::<f32>() { return el.gap(px(n)); }
///                 }
///                 // ... 其余前缀 ...
///                 el
///             }
///         }
///     }
///     let __class_str: &str = __class_expr.as_ref();
///     if __class_str.is_empty() { __el } else {
///         __class_str.split_ascii_whitespace().fold(__el, __rsx_apply_class)
///     }
/// }
/// ```
pub(crate) fn generate_dynamic_class_code_with_mode(
    class_expr: &syn::Expr,
    mode: ClassMode,
) -> TokenStream {
    let common_classes = get_cached_common_class_matches();
    let color_fallbacks = get_cached_color_fallback();
    let numeric_fallbacks = get_cached_numeric_fallback();
    let unknown_fallback = match mode {
        ClassMode::Permissive => quote! {
            // 仅在 debug 构建中打印警告，避免 release 中每帧触发 syscall 污染日志
            #[cfg(debug_assertions)]
            if !class.is_empty() {
                eprintln!(
                    "[gpui-rsx] warning: 动态 class {:?} 被忽略（不支持的 class 类型）\n  \
                     提示：改用字符串字面量 class=\"{}\" 可支持所有 class",
                    class, class
                );
            }
            el
        },
        ClassMode::Strict => quote! {
            panic!(
                "[gpui-rsx] unsupported dynamic class {:?} in strict mode. \
                 Use rsx! or rsx_permissive! to ignore unsupported dynamic classes.",
                class
            );
        },
    };

    quote! {
        {
            // match 表提取为 #[inline(never)] 局部函数：
            // - 阻止内联膨胀，同一组件内多个 class={expr} 共享函数体
            // - LLVM ICF 可合并同类型的单态化实例
            #[inline(never)]
            fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {
                match class {
                    #common_classes
                    _ => {
                        // 颜色前缀解析：覆盖完整 Tailwind 色板和 arbitrary hex。
                        #color_fallbacks
                        // 数值前缀回退：静态 match 未命中时，尝试前缀 + 数值解析
                        // 覆盖 gap-7、px-5、ml-3、opacity-33 等任意数值
                        #numeric_fallbacks
                        #unknown_fallback
                    }
                }
            }
            // AsRef<str>：&str、String、Cow<str> 均零拷贝通过
            let __class_expr = #class_expr;
            let __class_str: &str = __class_expr.as_ref();
            // 空字符串快速路径：跳过迭代器创建（常见于 class={if c { "flex" } else { "" }}）
            // split_ascii_whitespace 比 split_whitespace 更快——class 名只含 ASCII 字符
            if __class_str.is_empty() {
                __el
            } else {
                __class_str.split_ascii_whitespace().fold(__el, __rsx_apply_class)
            }
        }
    }
}

/// 生成动态颜色解析回退代码。
///
/// 相比为 text/bg/border 三个前缀展开完整色板 match arm，这里只在动态路径中
/// 解析 `prefix-family-shade`，把展开体积从 700+ 个颜色分支降到 22 个色系分支。
fn generate_color_fallback_code() -> TokenStream {
    let shade_arms = COLOR_SHADES.iter().enumerate().map(|(idx, shade)| {
        quote! { #shade => #idx, }
    });

    let family_arms = COLOR_FAMILIES.iter().map(|family| {
        let values = COLOR_SHADES.iter().map(|shade| {
            let key = format!("{family}_{shade}");
            lookup_color(&key).expect("COLOR_FAMILIES/COLOR_SHADES must match lookup_color")
        });
        quote! {
            #family => {
                const VALUES: [u32; 11] = [#(#values),*];
                Some(VALUES[shade_index])
            }
        }
    });

    quote! {
        fn __rsx_hex_digit(byte: u8) -> Option<u32> {
            match byte {
                b'0'..=b'9' => Some((byte - b'0') as u32),
                b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
                b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
                _ => None,
            }
        }

        fn __rsx_parse_hex_color(color: &str) -> Option<u32> {
            let inner = color.strip_prefix("[#")?.strip_suffix(']')?;
            let bytes = inner.as_bytes();
            match bytes.len() {
                8 => u32::from_str_radix(inner, 16).ok(),
                6 => u32::from_str_radix(inner, 16).ok(),
                4 => {
                    let r = __rsx_hex_digit(bytes[0])?;
                    let g = __rsx_hex_digit(bytes[1])?;
                    let b = __rsx_hex_digit(bytes[2])?;
                    let a = __rsx_hex_digit(bytes[3])?;
                    let expand = |n: u32| (n << 4) | n;
                    Some(expand(r) << 24 | expand(g) << 16 | expand(b) << 8 | expand(a))
                }
                3 => {
                    let r = __rsx_hex_digit(bytes[0])?;
                    let g = __rsx_hex_digit(bytes[1])?;
                    let b = __rsx_hex_digit(bytes[2])?;
                    Some(r << 20 | r << 16 | g << 12 | g << 8 | b << 4 | b)
                }
                _ => None,
            }
        }

        fn __rsx_parse_u8_component(raw: &str) -> Option<u8> {
            raw.trim().parse::<u8>().ok()
        }

        fn __rsx_parse_alpha_component(raw: &str) -> Option<u8> {
            let value = raw.trim().parse::<f32>().ok()?;
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return None;
            }
            Some((value * 255.0).round() as u8)
        }

        fn __rsx_parse_color_function(color: &str) -> Option<(u32, bool)> {
            let inner = color.strip_prefix('[')?.strip_suffix(']')?;

            if let Some(args) = inner.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
                let mut parts = args.split(',');
                let r = __rsx_parse_u8_component(parts.next()?)?;
                let g = __rsx_parse_u8_component(parts.next()?)?;
                let b = __rsx_parse_u8_component(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                return Some((((r as u32) << 16) | ((g as u32) << 8) | b as u32, false));
            }

            if let Some(args) = inner.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
                let mut parts = args.split(',');
                let r = __rsx_parse_u8_component(parts.next()?)?;
                let g = __rsx_parse_u8_component(parts.next()?)?;
                let b = __rsx_parse_u8_component(parts.next()?)?;
                let a = __rsx_parse_alpha_component(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                return Some((
                    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32,
                    true,
                ));
            }

            None
        }

        fn __rsx_parse_named_color(color: &str) -> Option<u32> {
            if color == "black" {
                return Some(0x000000);
            }
            if color == "white" {
                return Some(0xffffff);
            }

            let (family, shade) = color.rsplit_once('-')?;
            let shade_index = match shade {
                #(#shade_arms)*
                _ => return None,
            };

            match family {
                #(#family_arms,)*
                _ => None,
            }
        }

        fn __rsx_parse_color(color: &str) -> Option<(u32, bool)> {
            if let Some(hex) = __rsx_parse_hex_color(color) {
                let hex_inner = color.strip_prefix("[#")?.strip_suffix(']')?;
                let is_rgba = hex_inner.len() == 8 || hex_inner.len() == 4;
                return Some((hex, is_rgba));
            }
            __rsx_parse_color_function(color)
                .or_else(|| __rsx_parse_named_color(color).map(|color| (color, false)))
        }

        if let Some(rest) = class.strip_prefix("text-")
            && let Some((color, is_rgba)) = __rsx_parse_color(rest)
        {
            if is_rgba {
                return el.text_color(rgba(color));
            }
            return el.text_color(rgb(color));
        }
        if let Some(rest) = class.strip_prefix("bg-")
            && let Some((color, is_rgba)) = __rsx_parse_color(rest)
        {
            if is_rgba {
                return el.bg(rgba(color));
            }
            return el.bg(rgb(color));
        }
        if let Some(rest) = class.strip_prefix("border-")
            && let Some((color, is_rgba)) = __rsx_parse_color(rest)
        {
            if is_rgba {
                return el.border_color(rgba(color));
            }
            return el.border_color(rgb(color));
        }
    }
}

/// 生成数值前缀的回退匹配代码
///
/// 在静态 match 表未命中时，通过前缀识别 + `parse::<f32>()` 处理任意数值 class。
/// 每条 if-let 使用 `return` 提前返回；若全部未命中，执行流到达调用方的 `el`。
///
/// 优先检查较长前缀（`gap-x-` 先于 `gap-`），确保精确匹配：
/// `gap-x-4` 的 `strip_prefix("gap-")` 得 `"x-4"`，`parse::<f32>()` 失败，
/// 自然回退到 `gap-x-` 分支，无需额外排序。
fn generate_numeric_fallback_code() -> TokenStream {
    quote! {
        trait __RsxFiniteFloat {
            fn __rsx_finite(self) -> Result<f32, ()>;
        }

        impl __RsxFiniteFloat for Result<f32, std::num::ParseFloatError> {
            fn __rsx_finite(self) -> Result<f32, ()> {
                match self {
                    Ok(n) if n.is_finite() => Ok(n),
                    _ => Err(()),
                }
            }
        }

        // --- gap ---
        if let Some(rest) = class.strip_prefix("gap-x-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.gap_x(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.gap_x(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.gap_x(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("gap-y-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.gap_y(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.gap_y(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.gap_y(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("gap-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.gap(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.gap(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.gap(px(n)); }
        }
        // --- padding ---
        if let Some(rest) = class.strip_prefix("px-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.px(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.px(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.px(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("py-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.py(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.py(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.py(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pt-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pt(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pt(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.pt(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pb-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pb(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pb(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.pb(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pl-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pl(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pl(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.pl(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pr-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pr(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.pr(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.pr(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("p-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.p(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.p(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.p(px(n)); }
        }
        // --- margin ---
        if let Some(rest) = class.strip_prefix("mx-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mx(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mx(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.mx(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("my-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.my(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.my(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.my(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("mt-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mt(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mt(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.mt(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("mb-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mb(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mb(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.mb(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("ml-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.ml(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.ml(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.ml(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("mr-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mr(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.mr(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.mr(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("m-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.m(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.m(rems(n)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.m(px(n)); }
        }
        // --- sizing ---
        if let Some(rest) = class.strip_prefix("min-w-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.min_w(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.min_w(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.min_w(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.min_w(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.min_w(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("max-w-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.max_w(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.max_w(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.max_w(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.max_w(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.max_w(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("min-h-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.min_h(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.min_h(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.min_h(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.min_h(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.min_h(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("max-h-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.max_h(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.max_h(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.max_h(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.max_h(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.max_h(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("size-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.size(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.size(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.size(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.size(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.size(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("w-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.w(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.w(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.w(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.w(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.w(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("h-") {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.h(px(n)); }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.h(rems(n)); }
                }
                if let Some(raw) = inner.strip_suffix('%') {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.h(relative(n / 100.0)); }
                }
            }
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (num.parse::<f32>().__rsx_finite(), den.parse::<f32>().__rsx_finite()) {
                    if den > 0.0 { return el.h(relative(num / den)); }
                }
            }
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.h(px(n)); }
        }
        // --- opacity: opacity-50 → 0.50 ---
        if let Some(rest) = class.strip_prefix("opacity-") {
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() { return el.opacity(n / 100.0); }
        }
        // --- text layout ---
        if let Some(rest) = class.strip_prefix("line-clamp-") {
            if let Ok(n) = rest.parse::<usize>() { return el.line_clamp(n); }
        }
        // --- grid layout（优先检查较长前缀避免歧义）---
        if let Some(rest) = class.strip_prefix("col-span-") {
            if let Ok(n) = rest.parse::<u16>() { return el.col_span(n); }
        }
        if let Some(rest) = class.strip_prefix("col-start-") {
            if let Ok(n) = rest.parse::<i16>() { return el.col_start(n); }
        }
        if let Some(rest) = class.strip_prefix("col-end-") {
            if let Ok(n) = rest.parse::<i16>() { return el.col_end(n); }
        }
        if let Some(rest) = class.strip_prefix("row-span-") {
            if let Ok(n) = rest.parse::<u16>() { return el.row_span(n); }
        }
        if let Some(rest) = class.strip_prefix("row-start-") {
            if let Ok(n) = rest.parse::<i16>() { return el.row_start(n); }
        }
        if let Some(rest) = class.strip_prefix("row-end-") {
            if let Ok(n) = rest.parse::<i16>() { return el.row_end(n); }
        }
        if let Some(rest) = class.strip_prefix("grid-cols-") {
            if let Ok(n) = rest.parse::<u16>() { return el.grid_cols(n); }
        }
        if let Some(rest) = class.strip_prefix("grid-rows-") {
            if let Ok(n) = rest.parse::<u16>() { return el.grid_rows(n); }
        }
    }
}

/// 生成常用 class 的 match 分支
///
/// 返回一个 match arm 列表，每个 arm 匹配一个 class 字符串并应用相应的方法。
/// 通过 thread_local 缓存，整个编译过程只调用一次。
///
fn generate_common_class_matches() -> Vec<TokenStream> {
    // 非颜色静态工具类
    let static_classes = [
        // 布局
        "flex",
        "flex-col",
        "flex-col-reverse",
        "flex-row",
        "flex-row-reverse",
        "flex-1",
        "flex-auto",
        "flex-initial",
        "flex-none",
        "flex-grow",
        "flex-grow-0",
        "flex-wrap",
        "flex-wrap-reverse",
        "flex-nowrap",
        "flex-shrink",
        "flex-shrink-0",
        "block",
        "grid",
        "hidden",
        // 对齐
        "items-center",
        "items-start",
        "items-end",
        "items-baseline",
        "items-stretch",
        "justify-center",
        "justify-between",
        "justify-start",
        "justify-end",
        "justify-around",
        "justify-evenly",
        "content-normal",
        "content-center",
        "content-start",
        "content-end",
        "content-between",
        "content-around",
        "content-evenly",
        "content-stretch",
        "self-start",
        "self-end",
        "self-flex-start",
        "self-flex-end",
        "self-center",
        "self-baseline",
        "self-stretch",
        // 间距：gap
        "gap-1",
        "gap-2",
        "gap-3",
        "gap-4",
        "gap-5",
        "gap-6",
        "gap-8",
        "gap-10",
        "gap-12",
        // 间距：padding
        "p-1",
        "p-2",
        "p-3",
        "p-4",
        "p-5",
        "p-6",
        "p-8",
        "px-1",
        "px-2",
        "px-3",
        "px-4",
        "px-6",
        "py-1",
        "py-2",
        "py-3",
        "py-4",
        "py-6",
        "pt-1",
        "pt-2",
        "pt-4",
        "pt-6",
        "pb-1",
        "pb-2",
        "pb-4",
        "pb-6",
        "pl-2",
        "pl-4",
        "pr-2",
        "pr-4",
        // 间距：margin
        "m-1",
        "m-2",
        "m-4",
        "mx-1",
        "mx-2",
        "mx-4",
        "my-1",
        "my-2",
        "my-4",
        "mt-1",
        "mt-2",
        "mt-4",
        "mb-1",
        "mb-2",
        "mb-4",
        // 尺寸
        "w-full",
        "h-full",
        "size-full",
        "aspect-square",
        // 文本大小
        "text-xs",
        "text-sm",
        "text-base",
        "text-lg",
        "text-xl",
        "text-2xl",
        "text-3xl",
        // 文本对齐
        "text-left",
        "text-center",
        "text-right",
        // 文本装饰
        "whitespace-normal",
        "whitespace-nowrap",
        "truncate",
        "text-ellipsis",
        "text-ellipsis-start",
        "no-underline",
        "italic",
        "not-italic",
        "underline",
        "line-through",
        "text-decoration-solid",
        "text-decoration-wavy",
        "text-decoration-0",
        "text-decoration-1",
        "text-decoration-2",
        "text-decoration-4",
        "text-decoration-8",
        // 字体
        "font-thin",
        "font-extralight",
        "font-light",
        "font-normal",
        "font-medium",
        "font-semibold",
        "font-bold",
        "font-extrabold",
        "font-black",
        // 边框
        "border",
        "border-2",
        "border-dashed",
        "border-t",
        "border-b",
        "border-l",
        "border-r",
        "border-x",
        "border-y",
        "border-t-2",
        "border-b-2",
        "border-l-2",
        "border-r-2",
        "border-x-2",
        "border-y-2",
        "rounded-none",
        "rounded-sm",
        "rounded-md",
        "rounded-lg",
        "rounded-xl",
        "rounded-full",
        // 杂项
        "cursor-pointer",
        "cursor-default",
        "cursor-text",
        "cursor-move",
        "cursor-not-allowed",
        "cursor-context-menu",
        "cursor-crosshair",
        "cursor-vertical-text",
        "cursor-alias",
        "cursor-copy",
        "cursor-no-drop",
        "cursor-grab",
        "cursor-grabbing",
        "cursor-ew-resize",
        "cursor-ns-resize",
        "cursor-nesw-resize",
        "cursor-nwse-resize",
        "cursor-col-resize",
        "cursor-row-resize",
        "cursor-n-resize",
        "cursor-e-resize",
        "cursor-s-resize",
        "cursor-w-resize",
        "debug-outline",
        "overflow-hidden",
        "overflow-x-hidden",
        "overflow-y-hidden",
        "absolute",
        "relative",
        // 阴影
        "shadow-none",
        "shadow-2xs",
        "shadow-xs",
        "shadow-sm",
        "shadow-md",
        "shadow-lg",
        "shadow-xl",
        "shadow-2xl",
        // 透明度常用值（任意数值由数值前缀回退处理）
        "opacity-0",
        "opacity-25",
        "opacity-50",
        "opacity-75",
        "opacity-100",
        // grid placement
        "col-span-full",
        "col-start-auto",
        "col-end-auto",
        "row-span-full",
        "row-start-auto",
        "row-end-auto",
    ];

    let mut matches = Vec::with_capacity(static_classes.len());

    for class_str in static_classes {
        let method_call = parse_dynamic_common_class(class_str);
        matches.push(quote! {
            #class_str => #method_call,
        });
    }

    matches
}

fn parse_dynamic_common_class(class: &str) -> TokenStream {
    match class {
        "debug-outline" => quote! {
            {
                #[cfg(debug_assertions)]
                {
                    el.debug()
                }
                #[cfg(not(debug_assertions))]
                {
                    el
                }
            }
        },
        "flex-grow-0" => quote! {
            {
                let mut el = el;
                el.style().flex_grow = Some(0.0);
                el
            }
        },
        "items-stretch" => quote! {
            {
                let mut el = el;
                el.style().align_items = Some(AlignItems::Stretch);
                el
            }
        },
        "content-stretch" => quote! {
            {
                let mut el = el;
                el.style().align_content = Some(AlignContent::Stretch);
                el
            }
        },
        "justify-evenly" => quote! {
            {
                let mut el = el;
                el.style().justify_content = Some(JustifyContent::SpaceEvenly);
                el
            }
        },
        "self-start" | "self-flex-start" => quote! {
            {
                let mut el = el;
                el.style().align_self = Some(AlignItems::FlexStart);
                el
            }
        },
        "self-end" | "self-flex-end" => quote! {
            {
                let mut el = el;
                el.style().align_self = Some(AlignItems::FlexEnd);
                el
            }
        },
        "self-center" => quote! {
            {
                let mut el = el;
                el.style().align_self = Some(AlignItems::Center);
                el
            }
        },
        "self-baseline" => quote! {
            {
                let mut el = el;
                el.style().align_self = Some(AlignItems::Baseline);
                el
            }
        },
        "self-stretch" => quote! {
            {
                let mut el = el;
                el.style().align_self = Some(AlignItems::Stretch);
                el
            }
        },
        _ => {
            let method_call = parse_single_class_with_mode(class, ClassMode::Permissive);
            quote! { el #method_call }
        }
    }
}
