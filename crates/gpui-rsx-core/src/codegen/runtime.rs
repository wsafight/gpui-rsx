//! 运行时 class 处理
//!
//! 生成运行时动态 class 字符串解析和应用的代码。
//! 当 class 属性的值是表达式而非字符串字面量时使用。
//!
//! 优化：使用进程级缓存保存完整 helper 源码，避免多个动态 class 重复生成和分段解析。

use super::class::{ClassMode, SpacingScale, parse_single_class_with_mode};
use super::tables::{LENGTH_CLASS_SPECS, dynamic_common_classes};
mod color;
use color::generate_color_fallback_code;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::sync::OnceLock;
use syn::spanned::Spanned;

// 缓存所有 match 分支拼接后的字符串（每个进程和 spacing scale 只生成一次）
//
// 注意：不能缓存 proc_macro2::TokenStream，因为它的 token handle 绑定到
// 当前 proc macro 调用的 bridge 连接。每次调用结束后 bridge 失效，
// 下次调用时旧 handle 变成悬垂引用，导致 "use-after-free" panic。
//
// 优化：将完整 helper 缓存为单个字符串，每次宏调用只做 1 次 parse，
// 避免 common/color/numeric 三段代码分别重建和解析。
static PX_HELPER_STR: OnceLock<String> = OnceLock::new();
static TAILWIND_HELPER_STR: OnceLock<String> = OnceLock::new();

/// 获取完整动态 class helper（只与数值刻度有关，未知 class 的处理由调用点注入）。
///
/// 缓存字符串而非 `TokenStream`，避免跨 proc-macro bridge 保存失效的 token handle。
pub(crate) fn get_cached_dynamic_class_helper(mode: ClassMode) -> TokenStream {
    let cell = match mode.scale() {
        SpacingScale::Px => &PX_HELPER_STR,
        SpacingScale::Tailwind => &TAILWIND_HELPER_STR,
    };
    let source = cell.get_or_init(|| generate_dynamic_class_helper(mode).to_string());
    source
        .parse::<TokenStream>()
        .expect("cached dynamic class helper is valid")
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
/// 一次 RSX 宏调用内的所有动态 class 共享外层块中的一个泛型 helper。
/// 各属性仍保留自己的表达式求值、未知 class 回调和诊断位置。
pub(crate) fn generate_dynamic_class_code_with_mode(
    class_expr: &syn::Expr,
    mode: ClassMode,
    unknown_class_handler: Option<&syn::Expr>,
) -> TokenStream {
    let handler = match unknown_class_handler {
        Some(handler) => quote! { #handler },
        None => default_unknown_handler(mode.clone()),
    };

    let apply = if let Some(path) = mode.dynamic_runtime_path() {
        let helper = match mode.scale() {
            SpacingScale::Px => quote! { px },
            SpacingScale::Tailwind => quote! { tailwind },
        };
        quote! {
            let mut __el = __el;
            let __style = Styled::style(&mut __el);
            *__style = #path::#helper(std::mem::take(__style), __class_str, &mut __on_unknown);
            __el
        }
    } else {
        quote! {
            __class_str
                .split_ascii_whitespace()
                .fold(__el, |__el, __class| __rsx_apply_class(__el, __class, &mut __on_unknown))
        }
    };

    let result = quote! { if __class_str.is_empty() { __el } else { #apply } };
    let result = if let Some(edges) = super::format_class::static_edges(class_expr, mode.clone()) {
        let prefix = super::class::parse_class_string_with_mode(
            &edges.prefix,
            mode.clone(),
            class_expr.span(),
        );
        let suffix =
            super::class::parse_class_string_with_mode(&edges.suffix, mode, class_expr.span());
        let prefix_len = edges.prefix.len();
        let suffix_len = edges.suffix.len();
        quote! {
            let __class_str = &__class_str[#prefix_len..__class_str.len() - #suffix_len];
            let __el = __el #(#prefix)*;
            let __el = #result;
            __el #(#suffix)*
        }
    } else {
        result
    };

    quote! {
        {
            fn __rsx_unknown_class_handler<F: FnMut(&str)>(handler: F) -> F {
                handler
            }
            #[allow(unused_mut)]
            let mut __on_unknown = __rsx_unknown_class_handler(#handler);
            // AsRef<str>：&str、String、Cow<str> 均零拷贝通过
            let __class_expr = #class_expr;
            let __class_str: &str = __class_expr.as_ref();
            #result
        }
    }
}

fn default_unknown_handler(mode: ClassMode) -> TokenStream {
    if mode.is_strict() {
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

    let report = if mode.reports_dynamic_at_exit() {
        quote! {
            static __RSX_IGNORED: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
                std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));
            static __RSX_REGISTER: std::sync::Once = std::sync::Once::new();

            extern "C" fn __rsx_report_ignored_classes() {
                if let Ok(ignored) = __RSX_IGNORED.lock() {
                    if !ignored.is_empty() {
                        let mut names: Vec<_> = ignored.iter().cloned().collect();
                        names.sort();
                        eprintln!(
                            "[gpui-rsx] {} dynamic class(es) ignored at {}: {}",
                            ignored.len(),
                            concat!(file!(), ":", line!()),
                            names.join(", ")
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
                ignored.insert(__class.to_owned());
            }
        }
    } else {
        quote! {
            // 同一生成点的同一未知 class 只提示一次，避免 render loop 反复刷 stderr。
            static __RSX_WARNED: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
                std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));
            if let Ok(mut warned) = __RSX_WARNED.lock() {
                if warned.insert(__class.to_owned()) {
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
    dynamic_class_helper(
        mode,
        quote! { fn __rsx_apply_class<E: Styled>(el: E, class: &str, on_unknown: &mut dyn FnMut(&str)) -> E },
    )
}

/// Generates the optional crate-wide runtime without depending on a particular GPUI crate.
pub fn generate_runtime_module(gpui: &syn::Path) -> TokenStream {
    let helpers = [("px", SpacingScale::Px), ("tailwind", SpacingScale::Tailwind)]
        .into_iter()
        .map(|(name, scale)| {
            let name = syn::Ident::new(name, Span::call_site());
            let matcher = dynamic_class_helper(ClassMode::PERMISSIVE.with_scale(scale), quote! {
                fn __rsx_apply_class(el: StyleRefinement, class: &str, on_unknown: &mut dyn FnMut(&str)) -> StyleRefinement
            });
            quote! {
                #[doc(hidden)]
                pub fn #name(mut el: StyleRefinement, classes: &str, on_unknown: &mut dyn FnMut(&str)) -> StyleRefinement {
                    // The public `px` helper must not shadow GPUI's length constructor.
                    use #gpui::px;
                    #matcher
                    for class in classes.split_ascii_whitespace() {
                        el = __rsx_apply_class(el, class, on_unknown);
                    }
                    el
                }
            }
        });
    quote! {
        #[doc(hidden)]
        pub mod rsx_runtime {
            #[allow(unused_imports)]
            use #gpui::{prelude::*, *};
            #(#helpers)*
        }
    }
}

fn dynamic_class_helper(mode: ClassMode, signature: TokenStream) -> TokenStream {
    let common_classes = generate_common_class_matches(mode.clone());
    let color_fallbacks = generate_color_fallback_code();
    let numeric_fallbacks = generate_numeric_fallback_code(mode.scale());

    quote! {
        #[inline(never)]
        #signature {
            match class.as_bytes().first().copied() {
                #(#common_classes)*
                _ => {}
            }
            #color_fallbacks
            #numeric_fallbacks
            on_unknown(class);
            el
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
        if let Some(inner) = class.strip_prefix("text-[").and_then(|rest| rest.strip_suffix(']')) {
            if let Some(raw) = inner.strip_suffix("px") {
                if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.text_size(px(n)); }
            }
            if let Some(raw) = inner.strip_suffix("rem") {
                if let Ok(n) = raw.parse::<f32>().__rsx_finite() { return el.text_size(rems(n)); }
            }
        }
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
fn generate_common_class_matches(mode: ClassMode) -> Vec<TokenStream> {
    let mut groups = std::collections::BTreeMap::<u8, Vec<TokenStream>>::new();
    for class in dynamic_common_classes() {
        let method = parse_dynamic_common_class(class, mode.clone());
        groups
            .entry(class.as_bytes()[0])
            .or_default()
            .push(quote! { #class => return #method, });
    }
    groups
        .into_iter()
        .map(|(first, arms)| {
            quote! {
                Some(#first) => match class { #(#arms)* _ => {} },
            }
        })
        .collect()
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
                parse_single_class_with_mode(class, ClassMode::PERMISSIVE.with_scale(mode.scale()));
            quote! { el #method_call }
        }
    }
}
