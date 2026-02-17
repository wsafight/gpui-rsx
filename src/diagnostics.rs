//! 统一诊断模块
//!
//! 提供一致的错误消息和诊断助手函数

use syn::{Ident, spanned::Spanned};

/// 报告标签不匹配错误
pub fn tag_mismatch_error(closing_name: &Ident, opening_name: &Ident) -> syn::Error {
    syn::Error::new_spanned(
        closing_name,
        format!(
            "Closing tag `</{closing_name}>` does not match opening tag `<{opening_name}>`. \
             Tags must be properly nested.\n\
             \x20 help: Change the closing tag to `</{opening_name}>`\n\
             \x20 note: RSX syntax requires matching tags like in HTML/JSX"
        ),
    )
}

/// 报告未闭合标签错误
pub fn unclosed_tag_error(span: proc_macro2::Span, tag_name: &Ident) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "Unclosed tag `<{tag_name}>`. Expected closing tag before end of input.\n\
             \x20 help: Add a closing tag `</{tag_name}>`\n\
             \x20 note: All RSX tags must be properly closed"
        ),
    )
}

/// 报告未闭合 Fragment 错误
pub fn unclosed_fragment_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Unclosed fragment `<>`. Expected closing tag `</>` before end of input.\n\
         \x20 help: Add a closing tag `</>`\n\
         \x20 note: Fragments must be properly closed",
    )
}

/// 报告命名标签中的无效子节点错误
pub fn invalid_child_in_tag_error(span: proc_macro2::Span, tag_name: &Ident) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "Unexpected token in `<{tag_name}>`. \
             Expected one of: {{expr}}, \"text\", <child>, or </{tag_name}>\n\
             \x20 help: RSX children must be expressions in {{}}, text in quotes, or nested elements\n\
             \x20 note: Bare identifiers are not allowed - wrap them in braces like {{variable}}"
        ),
    )
}

/// 报告 Fragment 中的无效子节点错误
pub fn invalid_child_in_fragment_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Unexpected token in fragment. Expected one of: {expr}, \"text\", <child>, or </>\n\
         \x20 help: RSX children must be expressions in {}, text in quotes, or nested elements",
    )
}

/// 报告 for 循环缺少大括号错误
pub fn for_loop_missing_brace_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Expected '{' after for-in expression to start the loop body.\n\
         \x20 help: Add a block like: for item in items { <li>{item}</li> }\n\
         \x20 note: The for loop syntax is: for pattern in expression { body }",
    )
}

/// 报告 for 循环体内容无效错误
pub fn for_loop_invalid_body_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Unexpected token in for-loop body. Expected element, expression, or spread.\n\
         \x20 help: For-loop bodies must contain RSX elements like <div> or expressions like {item}\n\
         \x20 note: Example: for item in items { <li>{item}</li> }",
    )
}

/// 报告条件属性元组元素数量错误
pub fn condition_tuple_wrong_count_error<T: Spanned>(
    tuple: &T,
    attr_name: &str,
    found_count: usize,
) -> syn::Error {
    syn::Error::new(
        tuple.span(),
        format!(
            "The `{attr_name}` attribute expects exactly 2 values, found {found_count}.\n\
             \x20 help: Use the format: {attr_name}={{(condition, |el| el.method())}}\n\
             \x20 note: The first value is the condition, the second is a closure that modifies the element"
        ),
    )
}

/// 报告条件属性值类型错误
pub fn condition_tuple_wrong_type_error<T: Spanned>(value: &T, attr_name: &str) -> syn::Error {
    syn::Error::new(
        value.span(),
        format!(
            "The `{attr_name}` attribute expects a tuple of (condition, closure).\n\
             \x20 help: Use the format: {attr_name}={{(condition, |el| el.method())}}\n\
             \x20 note: Example: when={{(is_active, |el| el.bg(rgb(0x00ff00)))}}"
        ),
    )
}
