//! 运行时 class 处理
//!
//! 生成运行时动态 class 字符串解析和应用的代码。
//! 当 class 属性的值是表达式而非字符串字面量时使用。
//!
//! 优化：使用 thread_local 缓存 match 表，避免多个动态 class 重复生成相同 TokenStream。

use super::class::parse_single_class;
use super::tables::{COLOR_FAMILIES, COLOR_SHADES, lookup_color};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::cell::RefCell;

// 缓存所有 match 分支拼接后的字符串（thread_local 保证编译过程中只生成一次）
//
// 注意：不能缓存 proc_macro2::TokenStream，因为它的 token handle 绑定到
// 当前 proc macro 调用的 bridge 连接。每次调用结束后 bridge 失效，
// 下次调用时旧 handle 变成悬垂引用，导致 "use-after-free" panic。
//
// 优化：将所有 match arm 拼接为单个字符串，每次宏调用只做 1 次 parse，
// 而非对每个 arm 分别 parse（原先每次调用需 ~58 次 parse）。
thread_local! {
    static COMMON_CLASS_MATCHES_STR: RefCell<Option<String>> = const { RefCell::new(None) };
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

/// 生成运行时 class 解析代码
///
/// 当 class 属性是动态表达式时，生成一个在运行时解析和应用 class 的闭包。
///
/// # 支持的 class
///
/// 动态 class 支持两类解析：
/// 1. **静态 match 表**（快速路径）：[`generate_common_class_matches`] 中的预编译常用 class
/// 2. **数值前缀解析**（通用路径）：对间距/尺寸/透明度/层级类，支持任意数值
///    - `gap-7`、`gap-x-3`、`p-5`、`px-7`、`m-3`、`ml-5`、`w-48`、`h-16` 等
///    - `opacity-33`、`z-30` 等
///    - 静态 match 未命中时自动回退到此路径，无需扩充预编译列表
/// 3. **其余 class**（如颜色类 `text-orange-400`、自定义 class）仍需字符串字面量
///
/// 推荐方案（按性能从高到低）：
/// 1. **字符串字面量**（最佳）：`class="flex gap-4"` → 编译期展开，支持所有 class
/// 2. **条件表达式**（次佳）：`class={if active { "flex" } else { "block" }}`
/// 3. **动态表达式**：`class={dynamic_str}` → 支持间距/透明度/z-index 的任意数值
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
pub(crate) fn generate_dynamic_class_code(class_expr: &syn::Expr) -> TokenStream {
    let common_classes = get_cached_common_class_matches();
    let numeric_fallbacks = generate_numeric_fallback_code();

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
                        // 数值前缀回退：静态 match 未命中时，尝试前缀 + 数值解析
                        // 覆盖 gap-7、px-5、ml-3、opacity-33、z-30 等任意数值
                        #numeric_fallbacks
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
        // --- gap ---
        if let Some(rest) = class.strip_prefix("gap-x-") {
            if let Ok(n) = rest.parse::<f32>() { return el.gap_x(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("gap-y-") {
            if let Ok(n) = rest.parse::<f32>() { return el.gap_y(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("gap-") {
            if let Ok(n) = rest.parse::<f32>() { return el.gap(px(n)); }
        }
        // --- padding ---
        if let Some(rest) = class.strip_prefix("px-") {
            if let Ok(n) = rest.parse::<f32>() { return el.px(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("py-") {
            if let Ok(n) = rest.parse::<f32>() { return el.py(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pt-") {
            if let Ok(n) = rest.parse::<f32>() { return el.pt(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pb-") {
            if let Ok(n) = rest.parse::<f32>() { return el.pb(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pl-") {
            if let Ok(n) = rest.parse::<f32>() { return el.pl(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("pr-") {
            if let Ok(n) = rest.parse::<f32>() { return el.pr(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("p-") {
            if let Ok(n) = rest.parse::<f32>() { return el.p(px(n)); }
        }
        // --- margin ---
        if let Some(rest) = class.strip_prefix("mx-") {
            if let Ok(n) = rest.parse::<f32>() { return el.mx(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("my-") {
            if let Ok(n) = rest.parse::<f32>() { return el.my(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("mt-") {
            if let Ok(n) = rest.parse::<f32>() { return el.mt(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("mb-") {
            if let Ok(n) = rest.parse::<f32>() { return el.mb(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("ml-") {
            if let Ok(n) = rest.parse::<f32>() { return el.ml(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("mr-") {
            if let Ok(n) = rest.parse::<f32>() { return el.mr(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("m-") {
            if let Ok(n) = rest.parse::<f32>() { return el.m(px(n)); }
        }
        // --- sizing ---
        if let Some(rest) = class.strip_prefix("min-w-") {
            if let Ok(n) = rest.parse::<f32>() { return el.min_w(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("max-w-") {
            if let Ok(n) = rest.parse::<f32>() { return el.max_w(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("min-h-") {
            if let Ok(n) = rest.parse::<f32>() { return el.min_h(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("max-h-") {
            if let Ok(n) = rest.parse::<f32>() { return el.max_h(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("size-") {
            if let Ok(n) = rest.parse::<f32>() { return el.size(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("w-") {
            if let Ok(n) = rest.parse::<f32>() { return el.w(px(n)); }
        }
        if let Some(rest) = class.strip_prefix("h-") {
            if let Ok(n) = rest.parse::<f32>() { return el.h(px(n)); }
        }
        // --- opacity: opacity-50 → 0.50 ---
        if let Some(rest) = class.strip_prefix("opacity-") {
            if let Ok(n) = rest.parse::<f32>() { return el.opacity(n / 100.0); }
        }
        // --- z-index: z-10 → z_index(10) ---
        if let Some(rest) = class.strip_prefix("z-") {
            if let Ok(n) = rest.parse::<i32>() { return el.z_index(n); }
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
/// # 颜色覆盖
///
/// 通过复用编译期 `lookup_color()` 表，自动生成完整 Tailwind 色板的 match 分支：
/// - 22 色系 × 11 色阶 × 3 前缀（text / bg / border）= 726 条颜色分支
/// - 加上 black/white 的 6 条 = 共 732 条颜色分支
/// - 与静态工具类合并后，动态 class 几乎覆盖所有 Tailwind 颜色，大幅减少静默失败
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
        "flex-wrap",
        "flex-wrap-reverse",
        "flex-nowrap",
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
        "content-center",
        "content-start",
        "content-end",
        "content-between",
        "content-around",
        "content-evenly",
        "content-stretch",
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
        "truncate",
        "text-ellipsis",
        "italic",
        "not-italic",
        "underline",
        "line-through",
        // 字体
        "font-bold",
        // 边框
        "border",
        "border-2",
        "border-dashed",
        "rounded-sm",
        "rounded-md",
        "rounded-lg",
        "rounded-full",
        // 杂项
        "cursor-pointer",
        "overflow-hidden",
        "overflow-scroll",
        "absolute",
        "relative",
        // 透明度常用值（任意数值由数值前缀回退处理）
        "opacity-0",
        "opacity-25",
        "opacity-50",
        "opacity-75",
        "opacity-100",
        // Z 轴层级常用值
        "z-0",
        "z-10",
        "z-20",
        "z-50",
    ];

    // 22 色系 × 11 色阶 × 3 前缀 + 6 (black/white × 3 前缀)
    const COLOR_ENTRIES: usize = 22 * 11 * 3 + 6;
    let mut matches = Vec::with_capacity(static_classes.len() + COLOR_ENTRIES);

    for class_str in static_classes {
        let method_call = parse_single_class(class_str);
        matches.push(quote! {
            #class_str => el #method_call,
        });
    }

    // 自动生成完整 Tailwind 颜色 match 分支
    // 复用 tables::COLOR_FAMILIES / COLOR_SHADES 常量，真正单一数据源：
    // 新增色系只需修改 tables.rs，runtime.rs 自动同步。
    let families = COLOR_FAMILIES;
    let shades = COLOR_SHADES;

    // 复用 Ident，避免每次循环重复创建
    let text_color_ident = syn::Ident::new("text_color", Span::call_site());
    let bg_ident = syn::Ident::new("bg", Span::call_site());
    let border_color_ident = syn::Ident::new("border_color", Span::call_site());

    for family in families {
        for shade in shades {
            let color_key = format!("{family}_{shade}");
            if let Some(hex) = lookup_color(&color_key) {
                let text_class = format!("text-{family}-{shade}");
                let bg_class = format!("bg-{family}-{shade}");
                let border_class = format!("border-{family}-{shade}");
                matches.push(quote! { #text_class => el.#text_color_ident(rgb(#hex)), });
                matches.push(quote! { #bg_class => el.#bg_ident(rgb(#hex)), });
                matches.push(quote! { #border_class => el.#border_color_ident(rgb(#hex)), });
            }
        }
    }

    // black / white（tables.rs 中独立存储，单独生成）
    // 方法名直接编码在数据中，无需运行时 starts_with 分支判断
    for (class_str, method_ident, hex) in [
        ("text-black", &text_color_ident, 0x000000u32),
        ("text-white", &text_color_ident, 0xffffff_u32),
        ("bg-black", &bg_ident, 0x000000_u32),
        ("bg-white", &bg_ident, 0xffffff_u32),
        ("border-black", &border_color_ident, 0x000000_u32),
        ("border-white", &border_color_ident, 0xffffff_u32),
    ] {
        matches.push(quote! { #class_str => el.#method_ident(rgb(#hex)), });
    }

    matches
}
