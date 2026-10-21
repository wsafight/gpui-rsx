//! 运行时 class 处理
//!
//! 生成运行时动态 class 字符串解析和应用的代码。
//! 当 class 属性的值是表达式而非字符串字面量时使用。
//!
//! 优化：使用 thread_local 缓存完整 helper 源码，避免多个动态 class 重复生成和分段解析。

use super::class::{ClassMode, SpacingScale, parse_single_class_with_mode};
use super::tables::{
    COLOR_FAMILIES, COLOR_SHADES, LENGTH_CLASS_SPECS, dynamic_common_classes, lookup_color,
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::cell::OnceCell;

// 缓存所有 match 分支拼接后的字符串（thread_local 保证编译过程中只生成一次）
//
// 注意：不能缓存 proc_macro2::TokenStream，因为它的 token handle 绑定到
// 当前 proc macro 调用的 bridge 连接。每次调用结束后 bridge 失效，
// 下次调用时旧 handle 变成悬垂引用，导致 "use-after-free" panic。
//
// 优化：将完整 helper 缓存为单个字符串，每次宏调用只做 1 次 parse，
// 避免 common/color/numeric 三段代码分别重建和解析。
thread_local! {
    static PX_HELPER_STR: OnceCell<String> = const { OnceCell::new() };
    static TAILWIND_HELPER_STR: OnceCell<String> = const { OnceCell::new() };
}

/// 获取完整动态 class helper（只与数值刻度有关，未知 class 的处理由调用点注入）。
///
/// 缓存字符串而非 `TokenStream`，避免跨 proc-macro bridge 保存失效的 token handle。
fn get_cached_dynamic_class_helper(mode: ClassMode) -> TokenStream {
    let parse_cached = |cell: &OnceCell<String>| {
        let source = cell.get_or_init(|| generate_dynamic_class_helper(mode).to_string());
        source
            .parse::<TokenStream>()
            .expect("cached dynamic class helper is valid")
    };

    match mode.scale {
        SpacingScale::Px => PX_HELPER_STR.with(parse_cached),
        SpacingScale::Tailwind => TAILWIND_HELPER_STR.with(parse_cached),
    }
}

/// 生成运行时 class 解析代码
///
/// 当 class 属性是动态表达式时，生成一个在运行时解析和应用 class 的闭包。
///
/// 解析顺序：
/// 1. **静态 match 表**（快速路径）：GPUI 无参样式方法 + 常用语义 class
/// 2. **颜色前缀解析**：完整 Tailwind 色板 + arbitrary hex / rgb() / rgba()
/// 3. **数值前缀解析**：间距/尺寸/透明度/grid 等任意数值
/// 4. **未知 class**：交给 `on_unknown` 回调
///
/// 未知 class 的默认处理：宽松模式在 debug 构建中提示（逐条一次，或开启
/// `dynamic-class-report` 后在进程退出时汇总），严格模式 panic。
/// 元素上的 `onUnknownClass={|class| ...}` 可替换默认处理，避免生产环境崩溃。
///
/// # 代码体积优化
///
/// match 表被提取到 `#[inline(never)]` 泛型局部函数中：同一元素类型的多个
/// `class={expr}` 共享单态化实例，且不会内联膨胀父函数。
pub(crate) fn generate_dynamic_class_code_with_mode(
    class_expr: &syn::Expr,
    mode: ClassMode,
    unknown_class_handler: Option<&syn::Expr>,
) -> TokenStream {
    let helper = get_cached_dynamic_class_helper(mode);
    let handler = match unknown_class_handler {
        Some(handler) => quote! { #handler },
        None => default_unknown_handler(mode),
    };

    quote! {
        {
            #helper
            fn __rsx_unknown_class_handler<F: FnMut(&str)>(handler: F) -> F {
                handler
            }
            #[allow(unused_mut)]
            let mut __on_unknown = __rsx_unknown_class_handler(#handler);
            // AsRef<str>：&str、String、Cow<str> 均零拷贝通过
            let __class_expr = #class_expr;
            let __class_str: &str = __class_expr.as_ref();
            // 空字符串快速路径：跳过迭代器创建（常见于 class={if c { "flex" } else { "" }}）
            if __class_str.is_empty() {
                __el
            } else {
                __class_str
                    .split_ascii_whitespace()
                    .fold(__el, |__el, __class| __rsx_apply_class(__el, __class, &mut __on_unknown))
            }
        }
    }
}

fn default_unknown_handler(mode: ClassMode) -> TokenStream {
    if mode.strict {
        return quote! {
            |__class: &str| {
                panic!(
                    "[gpui-rsx] unsupported dynamic class {:?} in strict mode. \
                     Pass onUnknownClass={{|class| ...}} to handle it without panicking, \
                     or use rsx! / rsx_permissive! to ignore unsupported dynamic classes.",
                    __class
                );
            }
        };
    }

    let report = if mode.report_dynamic_at_exit {
        quote! {
            static __RSX_IGNORED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
            static __RSX_REGISTER: std::sync::Once = std::sync::Once::new();

            extern "C" fn __rsx_report_ignored_classes() {
                if let Ok(ignored) = __RSX_IGNORED.lock() {
                    if !ignored.is_empty() {
                        eprintln!(
                            "[gpui-rsx] {} dynamic class(es) ignored at {}: {}",
                            ignored.len(),
                            concat!(file!(), ":", line!()),
                            ignored.join(", ")
                        );
                    }
                }
            }

            __RSX_REGISTER.call_once(|| {
                unsafe extern "C" {
                    fn atexit(callback: extern "C" fn()) -> i32;
                }
                // SAFETY: registers a plain `extern "C"` function without captured state.
                unsafe {
                    atexit(__rsx_report_ignored_classes);
                }
            });
            if let Ok(mut ignored) = __RSX_IGNORED.lock() {
                if !ignored.iter().any(|seen| seen == __class) {
                    ignored.push(__class.to_owned());
                }
            }
        }
    } else {
        quote! {
            // 同一生成点的同一未知 class 只提示一次，避免 render loop 反复刷 stderr。
            static __RSX_WARNED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
            if let Ok(mut warned) = __RSX_WARNED.lock() {
                if !warned.iter().any(|seen| seen == __class) {
                    warned.push(__class.to_owned());
                    eprintln!(
                        "[gpui-rsx] warning: dynamic class {:?} ignored at {} (unsupported class)\n  \
                         hint: static literals such as class=\"{}\" are checked at compile time",
                        __class,
                        concat!(file!(), ":", line!()),
                        __class
                    );
                }
            }
        }
    };

    quote! {
        |__class: &str| {
            // 仅在 debug 构建中提示，避免 release 中每帧触发 syscall 污染日志。
            #[cfg(debug_assertions)]
            {
                #report
            }
            let _ = __class;
        }
    }
}

fn generate_dynamic_class_helper(mode: ClassMode) -> TokenStream {
    let common_classes = generate_common_class_matches(mode);
    let color_fallbacks = generate_color_fallback_code();
    let numeric_fallbacks = generate_numeric_fallback_code(mode.scale);

    quote! {
        #[inline(never)]
        fn __rsx_apply_class<E: Styled>(el: E, class: &str, on_unknown: &mut dyn FnMut(&str)) -> E {
            match class {
                #(#common_classes)*
                _ => {
                    // 颜色前缀解析：覆盖完整 Tailwind 色板和 arbitrary hex。
                    #color_fallbacks
                    // 数值前缀回退：gap-7、px-5、ml-3、opacity-33 等任意数值
                    #numeric_fallbacks
                    on_unknown(class);
                    el
                }
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

        // 生成代码不使用 let-chains：调用方 crate 可能仍是 edition 2021。
        if let Some(rest) = class.strip_prefix("text-") {
            if let Some((color, is_rgba)) = __rsx_parse_color(rest) {
                if is_rgba {
                    return el.text_color(rgba(color));
                }
                return el.text_color(rgb(color));
            }
        }
        if let Some(rest) = class.strip_prefix("bg-") {
            if let Some((color, is_rgba)) = __rsx_parse_color(rest) {
                if is_rgba {
                    return el.bg(rgba(color));
                }
                return el.bg(rgb(color));
            }
        }
        if let Some(rest) = class.strip_prefix("border-") {
            if let Some((color, is_rgba)) = __rsx_parse_color(rest) {
                if is_rgba {
                    return el.border_color(rgba(color));
                }
                return el.border_color(rgb(color));
            }
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
fn generate_numeric_fallback_code(scale: SpacingScale) -> TokenStream {
    let factor = scale.factor();
    let length_fallbacks = LENGTH_CLASS_SPECS
        .iter()
        .map(|spec| generate_length_fallback(spec, factor));
    let usize_fallbacks = [("line-clamp-", "line_clamp")]
        .into_iter()
        .map(|(prefix, method)| generate_integer_fallback(prefix, method, "usize"));
    let u16_fallbacks = [
        ("col-span-", "col_span"),
        ("row-span-", "row_span"),
        ("grid-cols-", "grid_cols"),
        ("grid-rows-", "grid_rows"),
    ]
    .into_iter()
    .map(|(prefix, method)| generate_integer_fallback(prefix, method, "u16"));
    let i16_fallbacks = [
        ("col-start-", "col_start"),
        ("col-end-", "col_end"),
        ("row-start-", "row_start"),
        ("row-end-", "row_end"),
    ]
    .into_iter()
    .map(|(prefix, method)| generate_integer_fallback(prefix, method, "i16"));

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

        #(#length_fallbacks)*
        // --- opacity: opacity-50 → 0.50 ---
        if let Some(rest) = class.strip_prefix("opacity-") {
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() {
                if (0.0..=100.0).contains(&n) {
                    return el.opacity(n / 100.0);
                }
            }
        }
        #(#usize_fallbacks)*
        #(#u16_fallbacks)*
        #(#i16_fallbacks)*
    }
}

fn generate_length_fallback(spec: &super::tables::LengthClassSpec, factor: f32) -> TokenStream {
    let prefix = spec.prefix;
    let method = syn::Ident::new(spec.method, Span::call_site());
    let percent = if spec.family.allows_percent() {
        quote! {
            if let Some(raw) = inner.strip_suffix('%') {
                if let Ok(n) = raw.parse::<f32>().__rsx_finite() {
                    return el.#method(relative(n / 100.0));
                }
            }
        }
    } else {
        quote! {}
    };
    let fraction = if spec.family.allows_fraction() {
        quote! {
            if let Some((num, den)) = rest.split_once('/') {
                if let (Ok(num), Ok(den)) = (
                    num.parse::<f32>().__rsx_finite(),
                    den.parse::<f32>().__rsx_finite(),
                ) {
                    if den > 0.0 {
                        return el.#method(relative(num / den));
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        if let Some(rest) = class.strip_prefix(#prefix) {
            if let Some(inner) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(raw) = inner.strip_suffix("px") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() {
                        return el.#method(px(n));
                    }
                }
                if let Some(raw) = inner.strip_suffix("rem") {
                    if let Ok(n) = raw.parse::<f32>().__rsx_finite() {
                        return el.#method(rems(n));
                    }
                }
                #percent
            }
            #fraction
            if let Ok(n) = rest.parse::<f32>().__rsx_finite() {
                return el.#method(px(n * #factor));
            }
        }
    }
}

fn generate_integer_fallback(prefix: &'static str, method: &'static str, ty: &str) -> TokenStream {
    let method = syn::Ident::new(method, Span::call_site());
    let ty: TokenStream = ty.parse().expect("integer fallback type is valid");

    quote! {
        if let Some(rest) = class.strip_prefix(#prefix) {
            if let Ok(n) = rest.parse::<#ty>() {
                return el.#method(n);
            }
        }
    }
}

/// 生成常用 class 的 match 分支
///
/// 返回一个 match arm 列表，每个 arm 匹配一个 class 字符串并应用相应的方法。
/// 通过 thread_local 缓存，整个编译过程只调用一次。
///
fn generate_common_class_matches(mode: ClassMode) -> impl Iterator<Item = TokenStream> {
    dynamic_common_classes().map(move |class_str| {
        let method_call = parse_dynamic_common_class(class_str, mode);
        quote! {
            #class_str => #method_call,
        }
    })
}

fn parse_dynamic_common_class(class: &str, mode: ClassMode) -> TokenStream {
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
        _ => {
            // 只用数值刻度；主题令牌需要渲染上下文，不进入动态 helper。
            let method_call =
                parse_single_class_with_mode(class, ClassMode::PERMISSIVE.with_scale(mode.scale));
            quote! { el #method_call }
        }
    }
}
