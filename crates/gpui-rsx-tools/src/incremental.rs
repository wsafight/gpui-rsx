//! Reuse macro boundaries when an edit is confined to one previously balanced body.
use crate::{MacroCall, RustAnalysis, nested_macro_calls};

pub(crate) fn calls(source: &str, old: &str, previous: &RustAnalysis) -> Option<Vec<MacroCall>> {
    let mut prefix = source
        .bytes()
        .zip(old.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !source.is_char_boundary(prefix) || !old.is_char_boundary(prefix) {
        prefix -= 1;
    }
    if source == old {
        return Some(previous.calls.clone());
    }
    let mut suffix = source[prefix..]
        .bytes()
        .rev()
        .zip(old[prefix..].bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    while !source.is_char_boundary(source.len() - suffix)
        || !old.is_char_boundary(old.len() - suffix)
    {
        suffix -= 1;
    }
    let old_end = old.len() - suffix;
    let outer = previous
        .calls
        .iter()
        .filter(|call| call.start <= prefix && old_end <= call.end)
        .max_by_key(|call| call.end - call.start)?;
    let delta = source.len() as isize - old.len() as isize;
    let end = outer.end.checked_add_signed(delta)?;
    let body = source.get(outer.start..end)?;
    // Wrapping also rejects a trailing line comment that would swallow the outer delimiter.
    let wrapped: proc_macro2::TokenStream = format!("{{{body}}}").parse().ok()?;
    if wrapped.into_iter().count() != 1 {
        return None;
    }
    let inner = nested_macro_calls(body).ok()?;
    let mut calls = Vec::new();
    for call in &previous.calls {
        if call == outer {
            calls.push(MacroCall {
                end,
                ..call.clone()
            });
            calls.extend(inner.iter().map(|call| MacroCall {
                name: call.name.clone(),
                start: outer.start + call.start,
                end: outer.start + call.end,
            }));
        } else if call.start >= outer.start && call.end <= outer.end {
            continue;
        } else if call.start > outer.end {
            calls.push(MacroCall {
                name: call.name.clone(),
                start: call.start.checked_add_signed(delta)?,
                end: call.end.checked_add_signed(delta)?,
            });
        } else {
            calls.push(call.clone());
        }
    }
    Some(calls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{analyze_rust_document, nested_macro_calls};
    use gpui_rsx_core::ClassMode;

    fn previous(source: &str) -> RustAnalysis {
        analyze_rust_document(source, ClassMode::PERMISSIVE)
    }

    #[test]
    fn unchanged_sources_reuse_the_previous_calls() {
        let source = "fn view(){ rsx!{<div/>}; }";
        let old = previous(source);
        let reused = calls(source, source, &old).unwrap();
        assert_eq!(reused, old.calls);
    }

    #[test]
    fn edits_inside_one_body_rescan_only_that_body() {
        let old = "fn view(){ rsx!{<div/>}; fn other(){} }";
        let analysis = previous(old);
        let changed = "fn view(){ rsx!{<div class=\"flex\"/>}; fn other(){} }";
        let reused = calls(changed, old, &analysis).unwrap();
        let full = nested_macro_calls(changed).unwrap();
        assert_eq!(reused, full);
    }

    #[test]
    fn edits_shift_later_calls_by_the_length_delta() {
        let old = "fn a(){ rsx!{<div/>}; rsx!{<span/>}; }";
        let analysis = previous(old);
        let changed = "fn a(){ rsx!{<div class=\"flex flex-col\"/>}; rsx!{<span/>}; }";
        let reused = calls(changed, old, &analysis).unwrap();
        assert_eq!(reused, nested_macro_calls(changed).unwrap());

        // Shrinking a body moves the second call backwards.
        let shrunk = "fn a(){ rsx!{<i/>}; rsx!{<span/>}; }";
        assert_eq!(
            calls(shrunk, changed, &previous(changed)).unwrap(),
            nested_macro_calls(shrunk).unwrap()
        );
    }

    #[test]
    fn structural_edits_fall_back_to_a_full_scan() {
        for (old, changed) in [
            ("fn view(){ rsx!{<div/>}; }", "fn view(){ other!{<div/>}; }"),
            ("fn view(){ rsx!{<div/>}; }", "fn view(){ /* moved */ }"),
            ("fn f(){}", "fn f(){ rsx!{<div/>}; }"),
        ] {
            let analysis = previous(old);
            assert!(
                calls(changed, old, &analysis).is_none(),
                "{old:?} -> {changed:?}"
            );
        }
    }

    #[test]
    fn a_trailing_line_comment_cannot_swallow_the_outer_delimiter() {
        let old = "fn view(){ rsx!{<div/>}; }";
        let changed = "fn view(){ rsx!{<div/> // trailing}; }";
        assert!(calls(changed, old, &previous(old)).is_none());
    }
}
