//! Keeps every hand-written Rust file under a line budget so modules stay reviewable.
//!
//! Counted: non-blank lines outside `#[cfg(test)]` items. Skipped: `tests/`, `benches/` and
//! `fuzz/` directories, `tests.rs` modules, and `@generated` files.
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;

pub const DEFAULT_MAX: usize = 600;

const ROOTS: &[&str] = &["src", "crates", "xtask/src", "editors/zed/src", "demo/src"];
const SKIPPED_DIRS: &[&str] = &["tests", "benches", "fuzz", "target", "node_modules"];

pub fn run(root: &Path, max: usize) -> Result<(), String> {
    let mut files = Vec::new();
    for dir in ROOTS {
        collect(&root.join(dir), &mut files)?;
    }
    files.sort();
    let mut over = Vec::new();
    for path in &files {
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if source
            .lines()
            .take(3)
            .any(|line| line.contains("@generated"))
        {
            continue;
        }
        let lines = counted_lines(&source).map_err(|e| format!("{}: {e}", path.display()))?;
        if lines > max {
            let relative = path.strip_prefix(root).unwrap_or(path);
            over.push(format!("  {lines:5} {}", relative.display()));
        }
    }
    if over.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{} file(s) exceed {max} non-test lines; split them by responsibility:\n{}",
        over.len(),
        over.join("\n")
    ))
}

fn collect(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        let name = path.file_name().and_then(|name| name.to_str());
        if path.extension().is_some_and(|ext| ext == "rs") && name != Some("tests.rs") {
            out.push(path.to_owned());
        }
        return Ok(());
    }
    for entry in std::fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let skipped = entry
            .file_name()
            .to_str()
            .is_some_and(|name| SKIPPED_DIRS.contains(&name));
        if !skipped {
            collect(&entry.path(), out)?;
        }
    }
    Ok(())
}

/// Non-blank lines outside items annotated `#[cfg(test)]`.
pub fn counted_lines(source: &str) -> Result<usize, String> {
    let file = syn::parse_file(source).map_err(|e| e.to_string())?;
    let mut excluded = Vec::new();
    for item in &file.items {
        let attrs = match item {
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Use(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Const(item) => &item.attrs,
            _ => continue,
        };
        if attrs.iter().any(is_cfg_test) {
            let span = item.span();
            excluded.push(span.start().line..=span.end().line);
        }
    }
    Ok(source
        .lines()
        .enumerate()
        .filter(|(index, line)| {
            !line.trim().is_empty() && !excluded.iter().any(|range| range.contains(&(index + 1)))
        })
        .count())
}

fn is_cfg_test(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("cfg")
        && attr
            .parse_args::<syn::Ident>()
            .is_ok_and(|ident| ident == "test")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("rsx-line-limit-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn test_items_and_blank_lines_are_not_counted() {
        let source = "fn a() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() { let _ = \"}\"; }\n}\n#[cfg(test)]\nfn helper() {}\nfn b() {}\n";
        assert_eq!(counted_lines(source).unwrap(), 2);
    }

    #[test]
    fn every_attributable_item_can_be_excluded() {
        let source = "\
#[cfg(test)]\nuse a::b;\n\
#[cfg(test)]\nstruct S;\n\
#[cfg(test)]\nconst C: u8 = 0;\n\
#[cfg(test)]\nimpl S {}\n\
#[cfg(test)]\nfn f() {}\n\
#[cfg(test)]\nmod m {}\n\
fn counted() {}\n";
        assert_eq!(counted_lines(source).unwrap(), 1);
    }

    #[test]
    fn cfg_not_test_is_still_counted() {
        let source = "#[cfg(feature = \"x\")]\nfn f() {}\n";
        assert_eq!(counted_lines(source).unwrap(), 2);
    }

    #[test]
    fn unparseable_sources_are_errors() {
        assert!(counted_lines("fn broken(").is_err());
    }

    #[test]
    fn missing_roots_are_ignored() {
        let root = temp_root("missing");
        assert_eq!(run(&root, DEFAULT_MAX), Ok(()));
        assert!(collect(&root.join("does-not-exist"), &mut Vec::new()).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn walks_sources_but_skips_tests_benches_and_generated_files() {
        let root = temp_root("walk");
        std::fs::create_dir_all(root.join("src/nested/tests")).unwrap();
        std::fs::create_dir_all(root.join("src/benches")).unwrap();
        std::fs::write(root.join("src/ok.rs"), "fn ok() {}\n").unwrap();
        std::fs::write(root.join("src/tests.rs"), "fn t() {}\n").unwrap();
        std::fs::write(root.join("src/nested/tests/skip.rs"), "fn t() {}\n").unwrap();
        std::fs::write(root.join("src/benches/skip.rs"), "fn b() {}\n").unwrap();
        std::fs::write(root.join("src/generated.rs"), "// @generated\nfn g() {}\n").unwrap();
        std::fs::write(root.join("src/big.rs"), "fn a() {}\nfn b() {}\nfn c() {}\n").unwrap();

        let error = run(&root, 2).unwrap_err();
        assert!(error.contains("src/big.rs"), "{error}");
        assert!(!error.contains("skip.rs"), "{error}");
        assert!(!error.contains("generated.rs"), "{error}");
        assert!(!error.contains("tests.rs"), "{error}");
        assert!(error.contains("exceed 2 non-test lines"), "{error}");

        std::fs::remove_file(root.join("src/big.rs")).unwrap();
        assert_eq!(run(&root, 2), Ok(()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
