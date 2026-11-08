use super::*;
use crate::{Diagnostic, DiagnosticSeverity};

#[test]
fn indexed_ownership_matches_innermost_containment_for_every_range() {
    let intervals = [
        (5, 75),
        (7, 20),
        (22, 66),
        (25, 32),
        (35, 61),
        (40, 43),
        (67, 70),
        (77, 79),
    ];
    let calls: Vec<_> = [3, 0, 4, 7, 2, 6, 1, 5]
        .into_iter()
        .map(|index| {
            let (start, end) = intervals[index];
            MacroCall {
                name: "rsx".to_owned(),
                start,
                end,
            }
        })
        .collect();
    let diagnostics = (0..=80)
        .flat_map(|start| {
            (start..=80).map(move |end| Diagnostic {
                start,
                end,
                severity: DiagnosticSeverity::Warning,
                message: "range".to_owned(),
                suggestion: None,
                code: None,
            })
        })
        .collect();
    let previous = RustAnalysis {
        calls,
        diagnostics,
        ..Default::default()
    };
    let source = "x".repeat(80);
    let index = ReuseIndex::new(&source, &previous);
    for (call, ranges) in previous.calls.iter().zip(&index.ranges) {
        let expected: Vec<_> = previous
            .diagnostics
            .iter()
            .enumerate()
            .filter_map(|(index, range)| {
                let owner = previous
                    .calls
                    .iter()
                    .filter(|candidate| {
                        range.start >= candidate.start && range.end <= candidate.end
                    })
                    .min_by_key(|candidate| candidate.end - candidate.start);
                (owner == Some(call)).then_some(index)
            })
            .collect();
        assert_eq!(ranges.diagnostics, expected, "{call:?}");
    }
}
