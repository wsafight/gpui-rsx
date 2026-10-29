//! Compile static edge tokens of format! templates while preserving the original formatting.
use syn::parse::{Parse, ParseStream};

pub(crate) struct Edges {
    pub prefix: String,
    pub suffix: String,
}

struct Arguments(syn::LitStr);
impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let literal = input.parse()?;
        let _: proc_macro2::TokenStream = input.parse()?;
        Ok(Self(literal))
    }
}

/// Only edge tokens have unambiguous byte positions: Display values may contain spaces
/// or copies of any static middle token. Keep the full format! call and slice its edges.
pub(crate) fn static_edges(expr: &syn::Expr, mode: super::class::ClassMode) -> Option<Edges> {
    let syn::Expr::Macro(expr) = expr else {
        return None;
    };
    // A qualified user macro could have unrelated semantics.
    if !expr.mac.path.is_ident("format")
        && !(expr.mac.path.segments.len() == 2
            && expr.mac.path.segments[0].ident == "std"
            && expr.mac.path.segments[1].ident == "format")
    {
        return None;
    }
    let args: Arguments = syn::parse2(expr.mac.tokens.clone()).ok()?;
    let template = args.0.value();
    let mut chars = template.chars().peekable();
    let mut decoded = String::new();
    let mut first = None;
    let mut last = 0;
    while let Some(c) = chars.next() {
        if (c == '{' || c == '}') && chars.peek() == Some(&c) {
            decoded.push(c);
            chars.next();
        } else if c == '{' {
            first.get_or_insert(decoded.len());
            loop {
                match chars.next()? {
                    '}' => break,
                    '{' => return None,
                    _ => {}
                }
            }
            decoded.push('\0');
            last = decoded.len();
        } else if c == '}' {
            return None;
        } else {
            decoded.push(c);
        }
    }
    let first = first?;
    let prefix_end = decoded[..first]
        .rfind(|c: char| c.is_ascii_whitespace())
        .map_or(0, |i| i + 1);
    let suffix_start = decoded[last..]
        .find(|c: char| c.is_ascii_whitespace())
        .map_or(decoded.len(), |i| last + i);
    let mut prefix = decoded[..prefix_end].to_owned();
    let mut suffix = decoded[suffix_start..].to_owned();
    let supported = |classes: &str| {
        classes.split_ascii_whitespace().all(|class| {
            matches!(
                super::class::analyze_class(class, mode.clone()).outcome,
                super::class::ClassOutcome::Supported(_)
            )
        })
    };
    if !supported(&prefix) {
        prefix.clear();
    }
    if !supported(&suffix) {
        suffix.clear();
    }
    if prefix.trim().is_empty() && suffix.trim().is_empty() {
        return None;
    }
    Some(Edges { prefix, suffix })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClassMode;

    fn edges(source: &str) -> Option<Edges> {
        let expr: syn::Expr = syn::parse_str(source).unwrap();
        static_edges(&expr, ClassMode::PERMISSIVE)
    }

    #[test]
    fn slices_static_tokens_around_display_values() {
        let left = edges(r#"format!("flex {}", value)"#).unwrap();
        assert_eq!(left.prefix, "flex ");
        assert_eq!(left.suffix, "");

        let right = edges(r#"format!("{} flex", value)"#).unwrap();
        assert_eq!(right.prefix, "");
        assert_eq!(right.suffix, " flex");

        let both = edges(r#"format!("flex {} items-center", value)"#).unwrap();
        assert_eq!(both.prefix, "flex ");
        assert_eq!(both.suffix, " items-center");

        let multi = edges(r#"format!("flex items-center {}", value)"#).unwrap();
        assert_eq!(multi.prefix, "flex items-center ");
        assert_eq!(multi.suffix, "");
    }

    #[test]
    fn accepts_the_std_qualified_macro() {
        assert!(edges(r#"std::format!("flex {}", value)"#).is_some());
    }

    #[test]
    fn ignores_unrelated_macros_and_plain_expressions() {
        for source in [
            r#"println!("flex {}", value)"#,
            r#"core::format!("flex {}", value)"#,
            r#"my::format!("flex {}", value)"#,
            "value",
        ] {
            assert!(edges(source).is_none(), "{source}");
        }
    }

    #[test]
    fn returns_none_when_no_edge_token_is_static() {
        for source in [
            r#"format!("{}", value)"#,
            r#"format!("{value}")"#,
            r#"format!("{}{}", a, b)"#,
            r#"format!("  {}", value)"#,
            r#"format!()"#,
        ] {
            assert!(edges(source).is_none(), "{source}");
        }
    }

    #[test]
    fn drops_edges_that_contain_unsupported_classes() {
        assert!(edges(r#"format!("flex-colum {}", value)"#).is_none());
        assert!(edges(r#"format!("{} flex-colum", value)"#).is_none());
        let edges = edges(r#"format!("flex-colum {} items-center", value)"#).unwrap();
        assert_eq!(edges.prefix, "");
        assert_eq!(edges.suffix, " items-center");
    }

    #[test]
    fn escaped_braces_are_ordinary_text() {
        // `{{}}` decodes to `{}`, which is not a supported class.
        assert!(edges(r#"format!("{{}} {}", value)"#).is_none());
        assert!(edges(r#"format!("{{flex}} {}", value)"#).is_none());
    }

    #[test]
    fn rejects_malformed_templates_and_arguments() {
        for source in [
            r#"format!("flex {", value)"#,
            r#"format!("flex } {}", value)"#,
            r#"format!("{ {x} }", value)"#,
            r#"format!(value)"#,
        ] {
            assert!(edges(source).is_none(), "{source}");
        }
    }
}
