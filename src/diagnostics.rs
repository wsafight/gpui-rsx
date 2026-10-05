//! Unified diagnostics module
//!
//! Provides consistent error messages and diagnostic helper functions

use syn::{Ident, spanned::Spanned};

/// Reports tag mismatch errors
pub fn tag_mismatch_error<T: Spanned>(
    closing_span: &T,
    closing_name: &str,
    opening_name: &str,
) -> syn::Error {
    syn::Error::new(
        closing_span.span(),
        format!(
            "Closing tag `</{closing_name}>` does not match opening tag `<{opening_name}>`. \
             Tags must be properly nested.\n\
             \x20 help: Change the closing tag to `</{opening_name}>`\n\
             \x20 note: RSX syntax requires matching tags like in HTML/JSX"
        ),
    )
}

/// Reports unclosed tag errors
pub fn unclosed_tag_error(span: proc_macro2::Span, tag_name: &str) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "Unclosed tag `<{tag_name}>`. Expected closing tag before end of input.\n\
             \x20 help: Add a closing tag `</{tag_name}>`\n\
             \x20 note: All RSX tags must be properly closed"
        ),
    )
}

/// Reports unclosed Fragment errors
pub fn unclosed_fragment_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Unclosed fragment `<>`. Expected closing tag `</>` before end of input.\n\
         \x20 help: Add a closing tag `</>`\n\
         \x20 note: Fragments must be properly closed",
    )
}

/// Reports invalid child node errors in named tags
pub fn invalid_child_in_tag_error(span: proc_macro2::Span, tag_name: &str) -> syn::Error {
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

/// Reports invalid child node errors in Fragment
pub fn invalid_child_in_fragment_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Unexpected token in fragment. Expected one of: {expr}, \"text\", <child>, or </>\n\
         \x20 help: RSX children must be expressions in {}, text in quotes, or nested elements",
    )
}

/// Reports missing brace errors in for loops
pub fn for_loop_missing_brace_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Expected '{' after for-in expression to start the loop body.\n\
         \x20 help: Add a block like: for item in items { <li>{item}</li> }\n\
         \x20 note: The for loop syntax is: for pattern in expression { body }",
    )
}

/// Reports invalid body errors in for loops
pub fn for_loop_invalid_body_error(span: proc_macro2::Span) -> syn::Error {
    syn::Error::new(
        span,
        "Unexpected token in for-loop body. Expected element, expression, or spread.\n\
         \x20 help: For-loop bodies must contain RSX elements like <div> or expressions like {item}\n\
         \x20 note: Example: for item in items { <li>{item}</li> }",
    )
}

/// Reports wrong element count errors in conditional attribute tuples
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

/// Reports missing `id` or `key` errors for elements with stateful attributes inside a for loop
///
/// GPUI requires IDs of all stateful elements in the same view to be globally unique. A for loop
/// expands the same code segment multiple times, causing multiple elements to share the same auto ID,
/// which leads to event routing and state management errors.
pub fn for_loop_missing_key_error<T: Spanned>(tag_span: &T, tag_name: &str) -> syn::Error {
    syn::Error::new(
        tag_span.span(),
        format!(
            "Element `<{tag_name}>` inside a for-loop has event handlers but no `id` or `key` attribute.\n\
             \x20 help: Add a `key={{unique_value}}` attribute so each iteration gets a unique ID:\n\
             \x20        for item in &self.items {{ <{tag_name} key={{item.id}} onClick={{...}}>...</{tag_name}> }}\n\
             \x20 note: Elements in loops share the same source location, so the auto-generated ID\n\
             \x20       would be identical across iterations, causing GPUI state conflicts.\n\
             \x20       Use `key` for a composite auto-ID, or `id` to supply a fully custom ID."
        ),
    )
}

/// Reports wrong type errors for conditional attribute values
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

/// Reports currently unsupported JSX-style attributes.
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

/// Reports GPUI generic attributes that cannot be expressed with current RSX attribute syntax.
pub fn unsupported_generic_attribute_error(attr_name: &Ident) -> syn::Error {
    syn::Error::new_spanned(
        attr_name,
        format!(
            "Unsupported generic GPUI attribute `{attr_name}`.\n\
             \x20 help: use `when` or `base` and call `group_drag_over::<YourType>(...)` explicitly\n\
             \x20 note: GPUI requires an explicit drag data type for `group_drag_over`, which RSX attributes cannot infer"
        ),
    )
}

/// Reports missing required construction attributes for GPUI tags.
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

/// Reports when the class parameter of `whenClass` is not a string literal.
pub fn when_class_string_literal_error<T: Spanned>(value: &T) -> syn::Error {
    syn::Error::new(
        value.span(),
        "The `whenClass` attribute expects a string literal as its second tuple value.\n\
         \x20 help: Use the format: whenClass={(condition, \"bg-blue-500 text-white\")}\n\
         \x20 note: Dynamic class strings are not supported by `whenClass`; use `when` for dynamic styling.",
    )
}

/// Reports stateful class inside `whenClass` error.
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

/// Reports when the class parameter of a state class attribute is not a string literal.
pub fn state_class_string_literal_error<T: Spanned>(attr_name: &Ident, value: &T) -> syn::Error {
    syn::Error::new(
        value.span(),
        format!(
            "The `{attr_name}` attribute expects a string literal class value.\n\
             \x20 help: Use the format: {attr_name}=\"bg-blue-500 text-white\"\n\
             \x20 note: State class attributes are compiled into GPUI style-refinement closures."
        ),
    )
}

/// Reports when a state class attribute contains a class not applicable to StyleRefinement.
pub fn state_class_unsupported_class_error(
    attr_name: &Ident,
    lit: &syn::LitStr,
    class: &str,
) -> syn::Error {
    syn::Error::new(
        lit.span(),
        format!(
            "Unsupported class `{class}` in `{attr_name}`.\n\
             \x20 help: Move `{class}` to `class=\"...\"`, or use `when` with an explicit GPUI method when it is conditional\n\
             \x20 note: `{attr_name}` maps to a GPUI StyleRefinement closure, so element-level classes such as `{class}` cannot be applied there"
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
        assert!(msg.contains("div"), "Should contain opening tag name div");
        assert!(msg.contains("span"), "Should contain closing tag name span");
    }

    #[test]
    fn tag_mismatch_contains_help_hint() {
        let closing = make_ident("div");
        let err = tag_mismatch_error(&closing, "div", "section");
        let msg = err.to_string();
        assert!(msg.contains("help:"), "Should contain help hint");
        assert!(msg.contains("note:"), "Should contain note hint");
    }

    // --- unclosed_tag_error ---

    #[test]
    fn unclosed_tag_contains_tag_name() {
        let tag = make_ident("nav");
        let err = unclosed_tag_error(Span::call_site(), &tag.to_string());
        let msg = err.to_string();
        assert!(msg.contains("nav"), "Should contain unclosed tag name");
        assert!(msg.contains("help:"), "Should contain help hint");
    }

    // --- unclosed_fragment_error ---

    #[test]
    fn unclosed_fragment_contains_help() {
        let err = unclosed_fragment_error(Span::call_site());
        let msg = err.to_string();
        assert!(msg.contains("</>"), "Should hint to close Fragment");
        assert!(msg.contains("help:"), "Should contain help hint");
    }

    // --- invalid_child_in_tag_error ---

    #[test]
    fn invalid_child_in_tag_contains_tag_and_hint() {
        let tag = make_ident("ul");
        let err = invalid_child_in_tag_error(Span::call_site(), &tag.to_string());
        let msg = err.to_string();
        assert!(msg.contains("ul"), "Should contain parent tag name");
        assert!(msg.contains("help:"), "Should contain help hint");
    }

    // --- for_loop_missing_brace_error ---

    #[test]
    fn for_loop_missing_brace_has_example() {
        let err = for_loop_missing_brace_error(Span::call_site());
        let msg = err.to_string();
        assert!(msg.contains("for"), "Should mention for loop");
        assert!(msg.contains("help:"), "Should contain help hint");
    }

    // --- condition_tuple_wrong_count_error ---

    #[test]
    fn condition_tuple_wrong_count_shows_found_and_expected() {
        // Simulate with a simple tuple
        let tokens: proc_macro2::TokenStream = "(a, b, c)".parse().unwrap();
        let expr: syn::ExprTuple = syn::parse2(tokens).unwrap();
        let err = condition_tuple_wrong_count_error(&expr, "when", 3);
        let msg = err.to_string();
        assert!(msg.contains("when"), "Should contain attribute name");
        assert!(msg.contains('3'), "Should contain actual element count");
        assert!(msg.contains("help:"), "Should contain help hint");
    }

    // --- condition_tuple_wrong_type_error ---

    #[test]
    fn condition_tuple_wrong_type_contains_attr_name() {
        let tokens: proc_macro2::TokenStream = "true".parse().unwrap();
        let expr: syn::Expr = syn::parse2(tokens).unwrap();
        let err = condition_tuple_wrong_type_error(&expr, "whenSome");
        let msg = err.to_string();
        assert!(msg.contains("whenSome"), "Should contain attribute name");
        assert!(msg.contains("help:"), "Should contain help hint");
    }

    #[test]
    fn unsupported_jsx_attribute_has_actionable_hint() {
        let attr = make_ident("whiteSpace");
        let err = unsupported_jsx_attribute_error(&attr);
        let msg = err.to_string();
        assert!(msg.contains("whiteSpace"), "Should contain attribute name");
        assert!(
            msg.contains("whitespace-nowrap"),
            "Should hint class syntax"
        );
        assert!(msg.contains("whitespace_nowrap"), "Should hint flag syntax");
    }

    #[test]
    fn unsupported_generic_attribute_has_actionable_hint() {
        let attr = make_ident("groupDragOver");
        let err = unsupported_generic_attribute_error(&attr);
        let msg = err.to_string();
        assert!(
            msg.contains("groupDragOver"),
            "Should contain attribute name"
        );
        assert!(msg.contains("group_drag_over::<YourType>"));
        assert!(msg.contains("cannot infer"));
    }

    #[test]
    fn when_class_string_literal_error_has_actionable_hint() {
        let tokens: proc_macro2::TokenStream = "dynamic_class".parse().unwrap();
        let expr: syn::Expr = syn::parse2(tokens).unwrap();
        let err = when_class_string_literal_error(&expr);
        let msg = err.to_string();
        assert!(msg.contains("whenClass"), "Should contain attribute name");
        assert!(msg.contains("string literal"), "Should hint string literal");
    }
}
