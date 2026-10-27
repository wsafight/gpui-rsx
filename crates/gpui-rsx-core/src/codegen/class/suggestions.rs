//! Completion vocabulary and bounded spelling suggestions.

use super::super::tables::*;
use super::variants::parse_variant;
use super::{ClassMode, SUPPORTED_VARIANTS, parse_length_number};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Vocabulary and suggestions
// ---------------------------------------------------------------------------

/// 所有已知的“具体” class 名（不含数值/任意值模式），用于拼写建议、补全与规范包。
pub fn known_classes() -> &'static [String] {
    static CLASSES: OnceLock<Vec<String>> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let mut out: Vec<String> = SEMANTIC_NO_ARG_CLASSES
            .iter()
            .map(|c| (*c).to_owned())
            .collect();
        for (name, _) in FONT_WEIGHTS {
            out.push((*name).to_owned());
        }
        for method in GPUI_METHODS {
            if !method.family_prefix.is_empty()
                && (method.negated
                    || !(method.family_prefix.starts_with("rounded")
                        || method.family_prefix.starts_with("border")
                        || matches!(method.family_suffix, "full" | "auto" | "px")))
            {
                continue;
            }
            let class = method.name.replace('_', "-");
            if gpui_no_arg_class_method(&class).is_some() && !out.contains(&class) {
                out.push(class);
            }
        }
        for prefix in ["text", "bg", "border"] {
            for family in COLOR_FAMILIES {
                for shade in COLOR_SHADES {
                    out.push(format!("{prefix}-{family}-{shade}"));
                }
            }
            out.push(format!("{prefix}-black"));
            out.push(format!("{prefix}-white"));
        }
        out.sort();
        out.dedup();
        out
    })
}

/// 拼写建议：编辑距离最近的已知 class（含长度前缀、状态前缀与主题令牌）。
pub fn suggest_class(class: &str, mode: ClassMode) -> Option<String> {
    if let Some((prefix, rest)) = class.split_once(':') {
        if parse_variant(prefix).is_err() {
            let variant = suggest(prefix, SUPPORTED_VARIANTS.iter().copied())?;
            return Some(format!("{variant}:{rest}"));
        }
        return suggest_class(rest, mode).map(|s| format!("{prefix}:{s}"));
    }

    // gpa-4 → gap-4：数值类只比较前缀
    if let Some((head, tail)) = class.rsplit_once('-')
        && (parse_length_number(tail).is_some() || tail.starts_with('['))
        && split_length_class(class).is_none()
    {
        let head = format!("{head}-");
        let prefixes = LENGTH_CLASS_SPECS.iter().map(|spec| spec.prefix);
        if let Some(prefix) = suggest(&head, prefixes) {
            return Some(format!("{prefix}{tail}"));
        }
    }

    let theme_classes = mode
        .theme
        .as_ref()
        .map(|theme| theme.class_names())
        .unwrap_or_default();
    let candidates = known_classes()
        .iter()
        .map(String::as_str)
        .chain(theme_classes.iter().map(String::as_str));
    suggest(class, candidates)
}

pub(super) fn suggest<'a>(
    input: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let max = match input.len() {
        0..=4 => 1,
        5..=10 => 2,
        _ => 3,
    };
    let mut best: Option<(usize, &str)> = None;
    // One distance table reused across all candidates instead of one allocation each.
    let mut table = Vec::new();
    for candidate in candidates {
        if candidate == input || candidate.len().abs_diff(input.len()) > max {
            continue;
        }
        let limit = best.map_or(max, |(d, _)| d.saturating_sub(1));
        let distance = bounded_distance(input, candidate, limit, &mut table);
        if distance <= limit && best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, candidate));
        }
    }
    best.map(|(_, c)| c.to_owned())
}

/// Optimal-string-alignment distance (Levenshtein plus adjacent transpositions, so
/// `gpa` → `gap` is 1). Returns `limit + 1` as soon as the distance must exceed `limit`.
fn bounded_distance(a: &str, b: &str, limit: usize, d: &mut Vec<usize>) -> usize {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let width = b.len() + 1;
    d.clear();
    d.resize((a.len() + 1) * width, 0);
    for i in 0..=a.len() {
        d[i * width] = i;
    }
    for (j, cell) in d.iter_mut().enumerate().take(width) {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (d[(i - 1) * width + j] + 1)
                .min(d[i * width + j - 1] + 1)
                .min(d[(i - 1) * width + j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[(i - 2) * width + j - 2] + 1);
            }
            d[i * width + j] = best;
        }
        // Every later cell is at least the minimum of the two latest rows.
        let row_min = d[i * width..(i + 1) * width]
            .iter()
            .min()
            .copied()
            .unwrap_or(0);
        let prev_min = d[(i - 1) * width..i * width]
            .iter()
            .min()
            .copied()
            .unwrap_or(0);
        if row_min.min(prev_min) > limit {
            return limit.saturating_add(1);
        }
    }
    d[a.len() * width + b.len()]
}
