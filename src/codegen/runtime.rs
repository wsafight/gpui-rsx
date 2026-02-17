//! 运行时 class 处理
//!
//! 生成运行时动态 class 字符串解析和应用的代码。
//! 当 class 属性的值是表达式而非字符串字面量时使用。
//!
//! 优化：使用 thread_local 缓存 match 表，避免多个动态 class 重复生成相同 TokenStream。

use super::class::parse_single_class;
use proc_macro2::TokenStream;
use quote::quote;
use std::cell::RefCell;
use std::rc::Rc;

// 缓存常用 class 的 match 分支（thread_local 保证编译过程中只生成一次）
// proc_macro2::TokenStream 不实现 Sync，无法使用 LazyLock，
// 但 proc macro 在单线程中执行，thread_local 是正确的缓存方案。
thread_local! {
    static COMMON_CLASS_MATCHES: RefCell<Option<Rc<Vec<TokenStream>>>> = const { RefCell::new(None) };
}

/// 获取缓存的 common class match 表（惰性初始化 + Rc 引用计数共享）
///
/// 使用 Rc 替代 Vec clone，将每次调用的拷贝成本从 O(n) 降为 O(1)。
fn get_cached_common_class_matches() -> Rc<Vec<TokenStream>> {
    COMMON_CLASS_MATCHES.with(|cell| {
        let mut borrow = cell.borrow_mut();
        Rc::clone(borrow.get_or_insert_with(|| Rc::new(generate_common_class_matches())))
    })
}

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
    let common_classes = get_cached_common_class_matches();
    let common_classes = &*common_classes;

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
/// 通过 thread_local 缓存，整个编译过程只调用一次。
fn generate_common_class_matches() -> Vec<TokenStream> {
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
        "px-4",
        "py-1",
        "py-2",
        "py-4",
        "m-2",
        "m-4",
        // 尺寸
        "w-full",
        "h-full",
        "size-full",
        // 文本
        "text-xs",
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
        "rounded-sm",
        "rounded-md",
        "rounded-lg",
        "rounded-full",
        // 其他
        "cursor-pointer",
        "overflow-hidden",
        "overflow-scroll",
        "absolute",
        "relative",
        // 高频颜色
        "bg-white",
        "bg-black",
        "text-white",
        "text-black",
    ];

    let mut matches = Vec::with_capacity(common_classes.len());
    for class_str in common_classes {
        let method_call = parse_single_class(class_str);
        matches.push(quote! {
            #class_str => el #method_call,
        });
    }

    matches
}
