//! Keep the documented compatibility matrix aligned with the demo lockfile.
use std::path::Path;

const START: &str = "<!-- gpui-rsx:compat:start -->";
const END: &str = "<!-- gpui-rsx:compat:end -->";

pub fn run(root: &Path, check: bool) -> Result<(), String> {
    let revs = crate::gen_tables::locked_revisions(root)?;
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let package = manifest
        .split("[package]")
        .nth(1)
        .ok_or("missing [package]")?;
    let version = package
        .lines()
        .find_map(|l| {
            l.strip_prefix("version = \"")
                .and_then(|s| s.strip_suffix('"'))
        })
        .ok_or("missing package version")?;
    let mut files = Vec::new();
    for (file, zh, anchor) in [
        ("README.md", false, "## 🚀 Quick Start"),
        ("README_CN.md", true, "## 🚀"),
        (
            "docs/src/content/docs/compatibility.md",
            false,
            "## Current Demo Matrix",
        ),
        (
            "docs/src/content/docs/zh-cn/compatibility.md",
            true,
            "## 当前 Demo 矩阵",
        ),
    ] {
        let path = root.join(file);
        let source = std::fs::read_to_string(&path).map_err(|e| format!("{file}: {e}"))?;
        let title = if zh {
            "### 已验证的兼容版本"
        } else {
            "### Verified Compatibility"
        };
        let note = if zh {
            "此矩阵来自 demo/Cargo.lock。Kit 统一提供 GPUI、组件和资源；以下 Cargo 别名保留 RSX 展开所需的 gpui:: 路径。请提交应用的 lockfile。"
        } else {
            "This matrix comes from demo/Cargo.lock. Kit supplies GPUI, components and assets; the Cargo alias below preserves the gpui:: path used by RSX expansions. Commit the application lockfile."
        };
        let block = format!(
            "{START}\n{title}\n\n| GPUI-RSX | GPUI Kit | GPUI snapshot | Component |\n| --- | --- | --- | --- |\n| `{version}` | `{}` | `{}` | `{}` |\n\n{note}\n\n```toml\n[dependencies]\ngpui = {{ package = \"gpui-kit\", version = \"={}\" }}\ngpui-rsx = \"{version}\"\n```\n{END}",
            revs.kit_version, revs.gpui, revs.component, revs.kit_version
        );
        let updated = replace_block(&source, &block, anchor).map_err(|e| format!("{file}: {e}"))?;
        files.push((path, updated));
    }
    crate::write_outputs(&files, check)
}

fn replace_block(source: &str, block: &str, anchor: &str) -> Result<String, String> {
    match (source.find(START), source.find(END)) {
        (Some(start), Some(end)) if start < end => Ok(format!(
            "{}{}{}",
            &source[..start],
            block,
            &source[end + END.len()..]
        )),
        (None, None) => {
            let at = source
                .find(anchor)
                .ok_or_else(|| format!("missing insertion anchor `{anchor}`"))?;
            Ok(format!("{}{}\n\n{}", &source[..at], block, &source[at..]))
        }
        _ => Err("incomplete compatibility markers".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_only_the_generated_block_and_rejects_broken_markers() {
        let block = format!("{START}\nnew\n{END}");
        let original = format!("before\n{START}\nold\n{END}\nafter");
        assert_eq!(
            replace_block(&original, &block, "unused").unwrap(),
            format!("before\n{block}\nafter")
        );
        assert!(replace_block(START, &block, "unused").is_err());
        assert!(replace_block("no anchor", &block, "missing").is_err());
    }
}
