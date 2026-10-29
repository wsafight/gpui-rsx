//! Completion context from tokens, including temporarily incomplete opening tags.
use crate::byte_offset;
use proc_macro2::{TokenStream, TokenTree};

#[derive(Debug, PartialEq, Eq)]
pub enum CompletionContext {
    Tag {
        start: usize,
        end: usize,
    },
    Attribute {
        tag: String,
        start: usize,
        end: usize,
    },
}

/// Positions are relative to one balanced macro body. Rust groups and literals are opaque.
pub fn context(source: &str, cursor: usize) -> Option<CompletionContext> {
    let tokens: TokenStream = source.parse().ok()?;
    let tokens: Vec<_> = tokens.into_iter().collect();
    let mut opening = None;
    for (i, token) in tokens.iter().enumerate() {
        let start = byte_offset(source, token.span(), false);
        let end = byte_offset(source, token.span(), true);
        if start > cursor {
            break;
        }
        if cursor > start && cursor < end {
            if let TokenTree::Group(group) = token {
                let inner: Vec<_> = group.stream().into_iter().collect();
                if matches!(inner.first(), Some(TokenTree::Ident(name)) if name == "for")
                    && let Some(TokenTree::Group(body)) = inner.last()
                    && body.delimiter() == proc_macro2::Delimiter::Brace
                {
                    let begin = byte_offset(source, body.span_open(), true);
                    let end = byte_offset(source, body.span_close(), false);
                    if (begin..=end).contains(&cursor) {
                        let mut context = context(&source[begin..end], cursor - begin)?;
                        match &mut context {
                            CompletionContext::Tag { start, end }
                            | CompletionContext::Attribute { start, end, .. } => {
                                *start += begin;
                                *end += begin;
                            }
                        }
                        return Some(context);
                    }
                }
                return None;
            }
            if matches!(token, TokenTree::Literal(_)) {
                return None;
            }
        }
        if let TokenTree::Punct(p) = token {
            match p.as_char() {
                '<' => opening = Some(i),
                '>' => opening = None,
                _ => {}
            }
        }
    }
    let index = opening?;
    let mut i = index + 1;
    if matches!(tokens.get(i), Some(TokenTree::Punct(p)) if p.as_char() == '/') {
        i += 1;
    }
    let delimiter_end = byte_offset(source, tokens[i - 1].span(), true);
    if cursor < delimiter_end {
        return None;
    }
    let start = tokens
        .get(i)
        .map_or(cursor, |t| byte_offset(source, t.span(), false));
    if start > cursor
        && !source
            .get(delimiter_end..cursor)?
            .chars()
            .all(char::is_whitespace)
    {
        return None;
    }
    let start = start.min(cursor);
    let mut end = start;
    while let Some(token) = tokens.get(i) {
        if matches!(token, TokenTree::Ident(_))
            || matches!(token, TokenTree::Punct(p) if p.as_char() == ':')
        {
            end = byte_offset(source, token.span(), true);
            i += 1;
            // A whitespace-separated identifier starts an attribute.
            if tokens
                .get(i)
                .is_some_and(|t| byte_offset(source, t.span(), false) > end)
            {
                break;
            }
        } else {
            break;
        }
    }
    if cursor <= end {
        return Some(CompletionContext::Tag { start, end });
    }
    let tag = source.get(start..end)?.to_owned();
    if tag.is_empty() {
        return Some(CompletionContext::Tag {
            start: cursor,
            end: cursor,
        });
    }
    let mut last_end = end;
    let mut after_equals = false;
    for token in &tokens[i..] {
        let at = byte_offset(source, token.span(), false);
        let to = byte_offset(source, token.span(), true);
        if at > cursor {
            break;
        }
        if matches!(token, TokenTree::Ident(_)) && cursor <= to && !after_equals {
            return Some(CompletionContext::Attribute {
                tag,
                start: at,
                end: to,
            });
        }
        after_equals = matches!(token, TokenTree::Punct(p) if p.as_char() == '=');
        last_end = to;
    }
    if after_equals
        || !source
            .get(last_end..cursor)?
            .chars()
            .all(char::is_whitespace)
    {
        return None;
    }
    Some(CompletionContext::Attribute {
        tag,
        start: cursor,
        end: cursor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(source: &str, needle: &str, offset: usize) -> Option<CompletionContext> {
        context(source, source.find(needle).unwrap() + offset)
    }

    #[test]
    fn complete_tags_report_their_name_range() {
        assert_eq!(
            at("<div", "div", 3),
            Some(CompletionContext::Tag { start: 1, end: 4 })
        );
        assert_eq!(
            at("</div", "div", 3),
            Some(CompletionContext::Tag { start: 2, end: 5 })
        );
        assert_eq!(
            at("<ui::Card", "Card", 4),
            Some(CompletionContext::Tag { start: 1, end: 9 })
        );
    }

    #[test]
    fn incomplete_tags_use_the_cursor_as_the_end() {
        // A cursor inside the tag name extends to the whole identifiable name.
        assert_eq!(
            at("<div", "div", 1),
            Some(CompletionContext::Tag { start: 1, end: 4 })
        );
        assert_eq!(
            context("<", 1),
            Some(CompletionContext::Tag { start: 1, end: 1 })
        );
    }

    #[test]
    fn attributes_report_their_name_range_after_a_tag() {
        assert_eq!(
            at("<div cla class=\"x\">", "cla", 3),
            Some(CompletionContext::Attribute {
                tag: "div".into(),
                start: 5,
                end: 8,
            })
        );
        assert_eq!(
            at("<div class=\"x\" id>", "id", 1),
            Some(CompletionContext::Attribute {
                tag: "div".into(),
                start: 15,
                end: 17,
            })
        );
    }

    #[test]
    fn empty_attribute_positions_insert_at_the_cursor() {
        let source = "<div class=\"x\" ";
        let cursor = source.len();
        assert_eq!(
            context(source, cursor),
            Some(CompletionContext::Attribute {
                tag: "div".into(),
                start: cursor,
                end: cursor,
            })
        );
    }

    #[test]
    fn values_and_rust_expressions_are_not_attribute_contexts() {
        for source in [
            "<div class= ",
            "<div id={foo.</",
            "<div class=\"fl",
            "plain text",
            "let x = 1;",
        ] {
            assert!(context(source, source.len()).is_none(), "{source}");
        }
    }

    #[test]
    fn loops_are_searched_at_their_inner_offset() {
        let source = "<div>{for item in items { <span cla /> }}</div>";
        assert_eq!(
            at(source, "cla", 3),
            Some(CompletionContext::Attribute {
                tag: "span".into(),
                start: source.find("cla").unwrap(),
                end: source.find("cla").unwrap() + 3,
            })
        );
    }

    #[test]
    fn comments_and_unbalanced_macros_have_no_context() {
        assert!(context("<div // comment", 8).is_none());
        assert!(context("<div", 0).is_none());
        assert!(context("</div", 1).is_none());
    }
}
