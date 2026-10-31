//! One command for every place that spells the release version.
//!
//! `cargo xtask bump <version>` rewrites manifests, lockfile entries of workspace crates and
//! install snippets in the docs, then refreshes the compatibility matrix. `--check` verifies
//! that all of them already match `[workspace.package].version`.
use std::path::{Path, PathBuf};

/// A version between `prefix` and `suffix`. `required` sites must exist, so a reworded
/// manifest fails loudly instead of silently keeping an old version.
#[derive(Clone)]
struct Site {
    prefix: String,
    suffix: &'static str,
    required: bool,
}

fn site(prefix: &str, suffix: &'static str) -> Site {
    Site {
        prefix: prefix.to_owned(),
        suffix,
        required: true,
    }
}

fn lock_entry(name: &str) -> Site {
    site(&format!("name = \"{name}\"\nversion = \""), "\"")
}

/// Install snippets and release links; historical examples use two-part versions such as
/// `gpui-rsx = "0.5"` and are never matched.
fn doc_sites() -> Vec<Site> {
    [
        ("gpui-rsx = \"", "\""),
        ("cargo install cargo-rsx --version ", " "),
        ("Install `cargo-rsx` ", " "),
        ("安装 `cargo-rsx` ", "（"),
        ("gpui-rsx-", ".vsix"),
        ("releases/tag/v", ")"),
        ("[", " release]"),
        ("[", " Release]"),
        ("release: '", " 已发布'"),
        ("release: '", " is here'"),
        ("GPUI-RSX <strong>", "</strong>"),
    ]
    .into_iter()
    .map(|(prefix, suffix)| Site {
        required: false,
        ..site(prefix, suffix)
    })
    .collect()
}

fn targets(root: &Path) -> Result<Vec<(PathBuf, Vec<Site>)>, String> {
    let mut targets = vec![
        (
            root.join("Cargo.toml"),
            vec![
                site("[workspace.package]\nversion = \"", "\""),
                site("gpui-rsx-core = { version = \"", "\""),
                site("gpui-rsx-tools = { version = \"", "\""),
            ],
        ),
        (
            root.join("editors/vscode/package.json"),
            vec![site("\"version\": \"", "\"")],
        ),
        (
            root.join("editors/zed/Cargo.toml"),
            vec![site("name = \"gpui-rsx-zed\"\nversion = \"", "\"")],
        ),
        (
            root.join("editors/zed/extension.toml"),
            vec![site("name = \"GPUI RSX\"\nversion = \"", "\"")],
        ),
        (
            root.join("docs/src/components/SiteTitle.astro"),
            vec![site("<span class=\"version\" translate=\"no\">", "</span>")],
        ),
        (
            root.join("Cargo.lock"),
            [
                "gpui-rsx",
                "gpui-rsx-core",
                "gpui-rsx-tools",
                "cargo-rsx",
                "gpui-rsx-wasm",
            ]
            .into_iter()
            .map(lock_entry)
            .collect(),
        ),
        (
            root.join("demo/Cargo.lock"),
            ["gpui-rsx", "gpui-rsx-core"]
                .into_iter()
                .map(lock_entry)
                .collect(),
        ),
        (
            root.join("editors/zed/Cargo.lock"),
            vec![lock_entry("gpui-rsx-zed")],
        ),
        (
            root.join("fuzz/Cargo.lock"),
            vec![lock_entry("gpui-rsx-core")],
        ),
    ];
    for fixture in ["binary-runtime", "shared-runtime", "gpui-0.2.2"] {
        let lockfile = root.join(format!("tests/fixtures/{fixture}/Cargo.lock"));
        if lockfile.is_file() {
            targets.push((
                lockfile,
                ["gpui-rsx", "gpui-rsx-core"]
                    .into_iter()
                    .map(lock_entry)
                    .collect(),
            ));
        }
    }
    let mut docs = vec![
        root.join("README.md"),
        root.join("README_CN.md"),
        root.join("editors/vscode/README.md"),
        root.join("editors/zed/README.md"),
        root.join("docs/src/components/HomeHero.astro"),
    ];
    collect_markdown(&root.join("docs/src/content/docs"), &mut docs)?;
    docs.sort();
    let sites = doc_sites();
    targets.extend(docs.into_iter().map(|doc| (doc, sites.clone())));
    Ok(targets)
}

fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            collect_markdown(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "md" || ext == "mdx")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn is_version(text: &str) -> bool {
    let (core, pre) = text.split_once('-').unwrap_or((text, ""));
    let parts: Vec<_> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        && pre.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.')
}

/// Replaces every version between `site.prefix` and `site.suffix`; returns the match count.
fn rewrite(text: &str, site: &Site, version: &str) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut count = 0;
    while let Some(start) = rest.find(site.prefix.as_str()) {
        let after = start + site.prefix.len();
        out.push_str(&rest[..after]);
        rest = &rest[after..];
        if let Some(end) = rest.find(site.suffix)
            && is_version(&rest[..end])
        {
            out.push_str(version);
            rest = &rest[end..];
            count += 1;
        }
    }
    out.push_str(rest);
    (out, count)
}

pub fn workspace_version(root: &Path) -> Result<String, String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let prefix = "[workspace.package]\nversion = \"";
    let start = manifest
        .find(prefix)
        .ok_or("missing [workspace.package] version")?
        + prefix.len();
    let end = manifest[start..]
        .find('"')
        .ok_or("unterminated workspace version")?;
    Ok(manifest[start..start + end].to_owned())
}

pub fn run(root: &Path, version: Option<&str>, check: bool) -> Result<(), String> {
    let version = match version {
        Some(version) if is_version(version) => version.to_owned(),
        Some(version) => return Err(format!("`{version}` is not a MAJOR.MINOR.PATCH version")),
        None => workspace_version(root)?,
    };
    let mut stale = Vec::new();
    let mut writes = Vec::new();
    for (path, sites) in targets(root)? {
        let original =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut text = original.clone();
        for site in &sites {
            let (updated, count) = rewrite(&text, site, &version);
            if site.required && count == 0 {
                return Err(format!(
                    "{}: no version after `{}`; update xtask/src/bump.rs",
                    path.display(),
                    site.prefix.escape_debug()
                ));
            }
            text = updated;
        }
        if text != original {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_owned();
            if check {
                stale.push(relative.display().to_string());
            } else {
                writes.push((path, relative, text));
            }
        }
    }
    // Write only after every required site matched so a failure leaves the tree untouched.
    for (path, relative, text) in writes {
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("updated {}", relative.display());
    }
    if !stale.is_empty() {
        return Err(format!(
            "these files do not use version {version}; run `cargo xtask bump {version}`:\n  {}",
            stale.join("\n  ")
        ));
    }
    crate::compat::run(root, check)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_only_three_part_versions_between_markers() {
        let doc = "gpui-rsx = \"0.9.0\"\ngpui-rsx = \"0.5\"\n[0.9.0 release](x/releases/tag/v0.9.0)\nKit <strong>0.8.1</strong>\n";
        let mut text = doc.to_owned();
        for site in &doc_sites() {
            text = rewrite(&text, site, "0.10.0").0;
        }
        assert_eq!(
            text,
            "gpui-rsx = \"0.10.0\"\ngpui-rsx = \"0.5\"\n[0.10.0 release](x/releases/tag/v0.10.0)\nKit <strong>0.8.1</strong>\n"
        );
        let lock = "name = \"gpui-rsx\"\nversion = \"0.9.0\"\n\nname = \"gpui-rsx-core\"\nversion = \"0.9.0\"\n";
        let (lock, count) = rewrite(lock, &lock_entry("gpui-rsx"), "0.10.0");
        assert_eq!(count, 1);
        assert!(lock.contains("name = \"gpui-rsx-core\"\nversion = \"0.9.0\""));
    }

    #[test]
    fn version_syntax_is_strict() {
        assert!(is_version("0.10.0") && is_version("1.0.0-beta.1"));
        assert!(!is_version("0.10") && !is_version("v0.10.0") && !is_version("0.10.0 "));
    }

    #[test]
    fn doc_sites_are_optional_and_cover_both_languages() {
        let sites = doc_sites();
        assert!(sites.iter().all(|site| !site.required));
        assert!(sites.iter().any(|site| site.prefix.contains("安装")));
        assert!(sites.iter().any(|site| site.prefix.contains("release: '")));
    }

    #[test]
    fn workspace_version_reads_only_the_workspace_section() {
        let dir = std::env::temp_dir().join(format!("rsx-bump-version-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nversion = \"0.1.0\"\n\n[workspace.package]\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        assert_eq!(workspace_version(&dir).unwrap(), "1.2.3");

        std::fs::write(dir.join("Cargo.toml"), "[package]\nversion = \"0.1.0\"\n").unwrap();
        assert!(workspace_version(&dir).unwrap_err().contains("missing"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn run_rejects_invalid_versions_before_touching_files() {
        let error = run(Path::new("."), Some("1.2"), false).unwrap_err();
        assert!(error.contains("not a MAJOR.MINOR.PATCH"), "{error}");
    }

    #[test]
    fn targets_allow_missing_generated_fixture_lockfiles() {
        let dir = std::env::temp_dir().join(format!("rsx-bump-targets-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("docs/src/content/docs")).unwrap();
        let fixture_dir = dir.join("tests/fixtures/binary-runtime");
        let lockfile = fixture_dir.join("Cargo.lock");
        let initial = targets(&dir).unwrap();
        assert!(!initial.iter().any(|(path, _)| path == &lockfile));
        assert!(
            initial
                .iter()
                .any(|(path, _)| path == &dir.join("fuzz/Cargo.lock"))
        );

        std::fs::create_dir_all(&fixture_dir).unwrap();
        std::fs::write(&lockfile, "").unwrap();
        let generated = targets(&dir).unwrap();
        assert!(generated.iter().any(|(path, _)| path == &lockfile));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rewrite_leaves_non_versions_and_missing_markers_alone() {
        let site = Site {
            prefix: "v".into(),
            suffix: "!",
            required: false,
        };
        let (text, count) = rewrite("v1.2.3! v1.2! vNext! v1.2.3-beta.1!", &site, "9.9.9");
        assert_eq!(count, 2);
        assert_eq!(text, "v9.9.9! v1.2! vNext! v9.9.9!");
        assert_eq!(
            rewrite("nothing", &site, "9.9.9"),
            ("nothing".to_owned(), 0)
        );
    }

    #[test]
    fn collect_markdown_walks_only_markdown_files() {
        let dir = std::env::temp_dir().join(format!("rsx-bump-docs-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("nested")).unwrap();
        std::fs::write(dir.join("a.md"), "").unwrap();
        std::fs::write(dir.join("nested/b.mdx"), "").unwrap();
        std::fs::write(dir.join("nested/c.txt"), "").unwrap();
        let mut out = Vec::new();
        collect_markdown(&dir, &mut out).unwrap();
        out.sort();
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|path| path.ends_with("a.md")));
        assert!(out.iter().any(|path| path.ends_with("b.mdx")));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
