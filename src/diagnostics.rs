//! 统一诊断模块
//!
//! 提供一致的错误消息和诊断助手函数

use syn::{Ident, spanned::Spanned};

/// 报告标签不匹配错误
pub fn tag_mismatch_error<T: Spanned>(
    closing_span: &T,
    closing_name: &str,
    opening_name: &str,
) -> syn::Error {
    syn::Error::new(
        closing_span.span(),
        format!(
            "Closing tag `</{}>` does not match opening tag `<{}>`. \
             Tags must be properly nested.\n\
             \x20 help: Change the closing tag to `</{}>`\n\
             \x20 note: RSX syntax requires matching tags like in HTML/JSX",
            closing_name, opening_name, opening_name
        ),
    )
}

/// 报告未闭合标签错误
pub fn unclosed_tag_error(span: proc_macro2::Span, tag_name: &str) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "Unclosed tag `<{}>`. Expected closing tag before end of input.\n\
             \x20 help: Add a closing tag `</{}>`\n\
             \x20 note: All RSX tags must be properly closed",
            tag_name, tag_name
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
pub fn invalid_child_in_tag_error(span: proc_macro2::Span, tag_name: &str) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "Unexpected token in `<{}>`. \
             Expected one of: {{expr}}, \"text\", <child>, or </{}>\n\
             \x20 help: RSX children must be expressions in {{}}, text in quotes, or nested elements\n\
             \x20 note: Bare identifiers are not allowed - wrap them in braces like {{variable}}",
            tag_name, tag_name
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
    if attr_name == "whenClass" {
        return syn::Error::new(
            tuple.span(),
            format!(
                "The `whenClass` attribute expects exactly 2 values, found {found_count}.\n\
                 \x20 help: Use the format: whenClass={{(condition, \"bg-blue-500 text-white\")}}\n\
                 \x20 note: The first value is the condition, the second is a static class string"
            ),
        );
    }

    syn::Error::new(
        tuple.span(),
        format!(
            "The `{attr_name}` attribute expects exactly 2 values, found {found_count}.\n\
             \x20 help: Use the format: {attr_name}={{(condition, |el| el.method())}}\n\
             \x20 note: The first value is the condition, the second is a closure that modifies the element"
        ),
    )
}

/// 报告 for 循环内含 stateful 属性的元素缺少 `id` 或 `key` 的错误
///
/// GPUI 要求同一视图中所有 stateful 元素的 ID 全局唯一。for 循环会将
/// 同一段代码展开多次，造成多个元素共享相同的自动 ID，导致事件路由和
/// 状态管理出现错误。
pub fn for_loop_missing_key_error<T: Spanned>(tag_span: &T, tag_name: &str) -> syn::Error {
    syn::Error::new(
        tag_span.span(),
        format!(
            "Element `<{}>` inside a for-loop has event handlers but no `id` or `key` attribute.\n\
             \x20 help: Add a `key={{unique_value}}` attribute so each iteration gets a unique ID:\n\
             \x20        for item in &self.items {{ <{} key={{item.id}} onClick={{...}}>...</{}> }}\n\
             \x20 note: Elements in loops share the same source location, so the auto-generated ID\n\
             \x20       would be identical across iterations, causing GPUI state conflicts.\n\
             \x20       Use `key` for a composite auto-ID, or `id` to supply a fully custom ID.",
            tag_name, tag_name, tag_name
        ),
    )
}

/// 报告条件属性值类型错误
pub fn condition_tuple_wrong_type_error<T: Spanned>(value: &T, attr_name: &str) -> syn::Error {
    if attr_name == "whenClass" {
        return syn::Error::new(
            value.span(),
            "The `whenClass` attribute expects a tuple of (condition, class_string).\n\
             \x20 help: Use the format: whenClass={(condition, \"bg-blue-500 text-white\")}\n\
             \x20 note: Dynamic class strings are not supported by `whenClass`; use `when` for dynamic styling.",
        );
    }

    syn::Error::new(
        value.span(),
        format!(
            "The `{attr_name}` attribute expects a tuple of (condition, closure).\n\
             \x20 help: Use the format: {attr_name}={{(condition, |el| el.method())}}\n\
             \x20 note: Example: when={{(is_active, |el| el.bg(rgb(0x00ff00)))}}"
        ),
    )
}

/// 报告暂不支持的 JSX 风格属性。
pub fn unsupported_jsx_attribute_error(attr_name: &Ident) -> syn::Error {
    syn::Error::new_spanned(
        attr_name,
        format!(
            "Unsupported JSX-style attribute `{attr_name}`.\n\
             \x20 help: use class=\"whitespace-nowrap\" or flag attribute `whitespace_nowrap`\n\
             \x20 note: GPUI exposes whitespace as dedicated flag methods instead of a string-valued property"
        ),
    )
}

/// 报告 GPUI 标签缺少构造必需属性。
pub fn missing_required_attribute_error<T: Spanned>(
    tag_span: &T,
    tag_name: &str,
    required: &str,
    example: &str,
) -> syn::Error {
    syn::Error::new(
        tag_span.span(),
        format!(
            "Element `<{tag_name}>` requires `{required}` for GPUI 0.2 construction.\n\
             \x20 help: Use the format: {example}\n\
             \x20 note: GPUI 0.2 does not expose a zero-argument `{tag_name}()` constructor"
        ),
    )
}

/// 报告 whenClass 的 class 参数不是字符串字面量。
pub fn when_class_string_literal_error<T: Spanned>(value: &T) -> syn::Error {
    syn::Error::new(
        value.span(),
        "The `whenClass` attribute expects a string literal as its second tuple value.\n\
         \x20 help: Use the format: whenClass={(condition, \"bg-blue-500 text-white\")}\n\
         \x20 note: Dynamic class strings are not supported by `whenClass`; use `when` for dynamic styling.",
    )
}

/// 报告 whenClass 中包含 stateful class。
pub fn when_class_stateful_error(lit: &syn::LitStr, class: &str) -> syn::Error {
    syn::Error::new(
        lit.span(),
        format!(
            "Unsupported stateful class `{class}` in `whenClass`.\n\
             \x20 help: Use `when` with an explicit closure for stateful classes\n\
             \x20 note: Stateful classes like `{class}` require element ID semantics that `whenClass` cannot provide"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    fn make_ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    // --- tag_mismatch_error ---

    #[test]
    fn tag_mismatch_contains_both_tag_names() {
        let closing = make_ident("span");
        let err = tag_mismatch_error(&closing, "span", "div");
        let msg = err.to_string();
        assert!(msg.contains("div"), "应包含开启标签名 div");
        assert!(msg.contains("span"), "应包含关闭标签名 span");
    }

    #[test]
    fn tag_mismatch_contains_help_hint() {
        let closing = make_ident("div");
        let err = tag_mismatch_error(&closing, "div", "section");
        let msg = err.to_string();
        assert!(msg.contains("help:"), "应包含 help 提示");
        assert!(msg.contains("note:"), "应包含 note 提示");
    }

    // --- unclosed_tag_error ---

    #[test]
    fn unclosed_tag_contains_tag_name() {
        let tag = make_ident("nav");
        let err = unclosed_tag_error(Span::call_site(), &tag.to_string());
        let msg = err.to_string();
        assert!(msg.contains("nav"), "应包含未闭合标签名");
        assert!(msg.contains("help:"), "应包含 help 提示");
    }

    // --- unclosed_fragment_error ---

    #[test]
    fn unclosed_fragment_contains_help() {
        let err = unclosed_fragment_error(Span::call_site());
        let msg = err.to_string();
        assert!(msg.contains("</>"), "应提示关闭 Fragment");
        assert!(msg.contains("help:"), "应包含 help 提示");
    }

    // --- invalid_child_in_tag_error ---

    #[test]
    fn invalid_child_in_tag_contains_tag_and_hint() {
        let tag = make_ident("ul");
        let err = invalid_child_in_tag_error(Span::call_site(), &tag.to_string());
        let msg = err.to_string();
        assert!(msg.contains("ul"), "应包含父标签名");
        assert!(msg.contains("help:"), "应包含 help 提示");
    }

    // --- for_loop_missing_brace_error ---

    #[test]
    fn for_loop_missing_brace_has_example() {
        let err = for_loop_missing_brace_error(Span::call_site());
        let msg = err.to_string();
        assert!(msg.contains("for"), "应提及 for 循环");
        assert!(msg.contains("help:"), "应包含 help 提示");
    }

    // --- condition_tuple_wrong_count_error ---

    #[test]
    fn condition_tuple_wrong_count_shows_found_and_expected() {
        // 用一个简单元组模拟
        let tokens: proc_macro2::TokenStream = "(a, b, c)".parse().unwrap();
        let expr: syn::ExprTuple = syn::parse2(tokens).unwrap();
        let err = condition_tuple_wrong_count_error(&expr, "when", 3);
        let msg = err.to_string();
        assert!(msg.contains("when"), "应包含属性名");
        assert!(msg.contains('3'), "应包含实际元素数");
        assert!(msg.contains("help:"), "应包含 help 提示");
    }

    // --- condition_tuple_wrong_type_error ---

    #[test]
    fn condition_tuple_wrong_type_contains_attr_name() {
        let tokens: proc_macro2::TokenStream = "true".parse().unwrap();
        let expr: syn::Expr = syn::parse2(tokens).unwrap();
        let err = condition_tuple_wrong_type_error(&expr, "whenSome");
        let msg = err.to_string();
        assert!(msg.contains("whenSome"), "应包含属性名");
        assert!(msg.contains("help:"), "应包含 help 提示");
    }

    #[test]
    fn unsupported_jsx_attribute_has_actionable_hint() {
        let attr = make_ident("whiteSpace");
        let err = unsupported_jsx_attribute_error(&attr);
        let msg = err.to_string();
        assert!(msg.contains("whiteSpace"), "应包含属性名");
        assert!(msg.contains("whitespace-nowrap"), "应提示 class 写法");
        assert!(msg.contains("whitespace_nowrap"), "应提示 flag 写法");
    }

    #[test]
    fn when_class_string_literal_error_has_actionable_hint() {
        let tokens: proc_macro2::TokenStream = "dynamic_class".parse().unwrap();
        let expr: syn::Expr = syn::parse2(tokens).unwrap();
        let err = when_class_string_literal_error(&expr);
        let msg = err.to_string();
        assert!(msg.contains("whenClass"), "应包含属性名");
        assert!(msg.contains("string literal"), "应提示字符串字面量");
    }
}
