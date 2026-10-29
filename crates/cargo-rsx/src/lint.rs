use super::{Options, is_stdin, mode, mode_path, sources};
use gpui_rsx_tools::DiagnosticSeverity;
use serde_json::json;

pub(super) fn run(opt: &Options) -> Result<i32, String> {
    let mut reports = Vec::new();
    let mut failed = false;
    for (path, source) in sources(opt)? {
        let mode = mode(opt, mode_path(&path))?;
        let diagnostics = if is_stdin(opt) && source.trim_start().starts_with('<') {
            gpui_rsx_tools::analyze_body(&source, mode)
        } else {
            gpui_rsx_tools::analyze_rust(&source, mode)
        };
        for diagnostic in diagnostics {
            failed |= match diagnostic.severity {
                DiagnosticSeverity::Error => true,
                DiagnosticSeverity::Warning => opt.deny_warnings,
                // Information and any future, milder severities never fail a lint.
                _ => false,
            };
            let rank = |s| match s {
                DiagnosticSeverity::Error => 0,
                DiagnosticSeverity::Warning => 1,
                _ => 2,
            };
            let maximum = if opt.quiet {
                0
            } else {
                match opt.min_severity.as_deref() {
                    Some("error") => 0,
                    Some("warning") => 1,
                    _ => 2,
                }
            };
            if rank(diagnostic.severity) > maximum {
                continue;
            }
            let line = source[..diagnostic.start]
                .bytes()
                .filter(|&b| b == b'\n')
                .count()
                + 1;
            // Columns count characters, as editors and rustc do, not bytes.
            let line_start = source[..diagnostic.start].rfind('\n').map_or(0, |n| n + 1);
            let column = source[line_start..diagnostic.start].chars().count() + 1;
            if !opt.json {
                println!(
                    "{}:{line}:{column}: {}: {}",
                    path.display(),
                    diagnostic.severity,
                    diagnostic.message
                );
            }
            reports.push(
                json!({"file": path, "line": line, "column": column, "diagnostic": diagnostic}),
            );
        }
    }
    if opt.json {
        println!("{}", json!({"ok": !failed, "diagnostics": reports}));
    }
    Ok(i32::from(failed))
}
