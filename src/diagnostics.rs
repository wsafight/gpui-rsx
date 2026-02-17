//! 统一诊断模块
//!
//! 提供一致的错误消息和诊断助手函数

use proc_macro_error2::abort;
use syn::{Ident, spanned::Spanned};

/// 报告标签不匹配错误
pub fn tag_mismatch_error(closing_name: &Ident, opening_name: &Ident) -> ! {
    abort!(
        closing_name,
        "Closing tag `</{}>` does not match opening tag `<{}>`. Tags must be properly nested.",
        closing_name, opening_name;
        help = "Change the closing tag to `</{}>`", opening_name;
        note = "RSX syntax requires matching tags like in HTML/JSX"
    );
}

/// 报告未闭合标签错误
pub fn unclosed_tag_error(span: proc_macro2::Span, tag_name: &Ident) -> ! {
    abort!(
        span,
        "Unclosed tag `<{}>`. Expected closing tag before end of input.", tag_name;
        help = "Add a closing tag `</{}>`", tag_name;
        note = "All RSX tags must be properly closed"
    );
}

/// 报告未闭合 Fragment 错误
pub fn unclosed_fragment_error(span: proc_macro2::Span) -> ! {
    abort!(
        span,
        "Unclosed fragment `<>`. Expected closing tag `</>` before end of input.";
        help = "Add a closing tag `</>`";
        note = "Fragments must be properly closed"
    );
}

/// 报告命名标签中的无效子节点错误
pub fn invalid_child_in_tag_error(span: proc_macro2::Span, tag_name: &Ident) -> ! {
    abort!(
        span,
        "Unexpected token in `<{}>`. Expected one of: {{expr}}, \"text\", <child>, or </{}>",
        tag_name, tag_name;
        help = "RSX children must be expressions in {{}}, text in quotes, or nested elements";
        note = "Bare identifiers are not allowed - wrap them in braces like {{variable}}"
    );
}

/// 报告 Fragment 中的无效子节点错误
pub fn invalid_child_in_fragment_error(span: proc_macro2::Span) -> ! {
    abort!(
        span,
        "Unexpected token in fragment. Expected one of: {{expr}}, \"text\", <child>, or </>";
        help = "RSX children must be expressions in {{}}, text in quotes, or nested elements"
    );
}

/// 报告 for 循环缺少大括号错误
pub fn for_loop_missing_brace_error(span: proc_macro2::Span) -> ! {
    abort!(
        span,
        "Expected '{{' after for-in expression to start the loop body.";
        help = "Add a block like: for item in items {{ <li>{{item}}</li> }}";
        note = "The for loop syntax is: for pattern in expression {{ body }}"
    );
}

/// 报告 for 循环体内容无效错误
pub fn for_loop_invalid_body_error(span: proc_macro2::Span) -> ! {
    abort!(
        span,
        "Unexpected token in for-loop body. Expected element, expression, or spread.";
        help = "For-loop bodies must contain RSX elements like <div> or expressions like {{item}}";
        note = "Example: for item in items {{ <li>{{item}}</li> }}"
    );
}

/// 报告条件属性元组元素数量错误
pub fn condition_tuple_wrong_count_error<T: Spanned>(
    tuple: &T,
    attr_name: &str,
    found_count: usize,
) -> ! {
    abort!(
        tuple.span(),
        "The `{}` attribute expects exactly 2 values, found {}.", attr_name, found_count;
        help = "Use the format: {}={{(condition, |el| el.method())}}", attr_name;
        note = "The first value is the condition, the second is a closure that modifies the element"
    );
}

/// 报告条件属性值类型错误
pub fn condition_tuple_wrong_type_error<T: Spanned>(value: &T, attr_name: &str) -> ! {
    abort!(
        value.span(),
        "The `{}` attribute expects a tuple of (condition, closure).", attr_name;
        help = "Use the format: {}={{(condition, |el| el.method())}}", attr_name;
        note = "Example: when={{(is_active, |el| el.bg(rgb(0x00ff00)))}}"
    );
}
