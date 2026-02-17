//! 运行时 class 处理
//!
//! 生成运行时动态 class 字符串解析和应用的代码。
//! 当 class 属性的值是表达式而非字符串字面量时使用。

use super::class::parse_single_class;
use proc_macro2::TokenStream;
use quote::quote;

/// 生成运行时 class 解析代码
///
/// 当 class 属性是动态表达式时，生成一个在运行时解析和应用 class 的闭包。
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
///             _ => el,
///         }
///     }
///     let __class_expr = <expression>;
///     let __class_str: &str = __class_expr.as_ref();
///     __class_str.split_whitespace().fold(__el, __rsx_apply_class)
/// }
/// ```
///
/// # 性能说明
///
/// - 运行时解析比编译期解析慢
/// - 建议优先使用字符串字面量以获得最佳性能
/// - 对于少量条件变化，使用条件表达式: `if active { "flex" } else { "block" }`
pub(crate) fn generate_dynamic_class_code(class_expr: &syn::Expr) -> TokenStream {
    let common_classes = generate_common_class_matches();

    quote! {
        {
            // match 表提取为 #[inline(never)] 局部函数：
            // - 阻止内联膨胀，同一组件内多个 class={expr} 共享函数体
            // - LLVM ICF 可合并同类型的单态化实例
            #[inline(never)]
            fn __rsx_apply_class<E: Styled>(el: E, class: &str) -> E {
                match class {
                    #(#common_classes)*
                    _ => el,
                }
            }
            // AsRef<str>：&str、String、Cow<str> 均零拷贝通过
            let __class_expr = #class_expr;
            let __class_str: &str = __class_expr.as_ref();
            __class_str.split_whitespace().fold(__el, __rsx_apply_class)
        }
    }
}

/// 生成常用 class 的 match 分支
///
/// 返回一个 match arm 列表，每个 arm 匹配一个 class 字符串并应用相应的方法。
fn generate_common_class_matches() -> Vec<TokenStream> {
    let mut matches = Vec::new();

    // 常用的 class 列表（覆盖最常见的用例）
    let common_classes = [
        // 布局
        "flex",
        "flex-col",
        "flex-row",
        "flex-1",
        "flex-wrap",
        // 对齐
        "items-center",
        "items-start",
        "items-end",
        "justify-center",
        "justify-between",
        "justify-start",
        "justify-end",
        // 间距（预设值）
        "gap-2",
        "gap-4",
        "gap-6",
        "p-2",
        "p-4",
        "px-2",
        "px-4",
        "py-2",
        "py-4",
        "m-2",
        "m-4",
        // 尺寸
        "w-full",
        "h-full",
        "size-full",
        // 文本
        "text-sm",
        "text-base",
        "text-lg",
        "text-xl",
        "text-2xl",
        "text-3xl",
        "font-bold",
        // 边框
        "border",
        "border-2",
        "rounded-md",
        "rounded-lg",
        "rounded-full",
        // 其他
        "cursor-pointer",
        "overflow-hidden",
        "absolute",
        "relative",
    ];

    for class_str in common_classes {
        let method_call = parse_single_class(class_str);
        matches.push(quote! {
            #class_str => el #method_call,
        });
    }

    matches
}
