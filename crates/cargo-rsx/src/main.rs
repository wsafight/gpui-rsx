mod doctor;
mod lsp;
mod preview;

use gpui_rsx_core::{ClassMode, SpacingScale, Theme};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Default)]
struct Options {
    manifest: Option<PathBuf>,
    path: Option<PathBuf>,
    output: Option<PathBuf>,
    json: bool,
    strict: bool,
    tailwind: bool,
    deny_warnings: bool,
    check: bool,
    write: bool,
    watch: bool,
    offline: bool,
    index: usize,
    port: u16,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("cargo-rsx: {error}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32, String> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "rsx") {
        args.remove(0);
    }
    let command = args
        .first()
        .map(String::as_str)
        .unwrap_or("help")
        .to_owned();
    if matches!(command.as_str(), "help" | "--help" | "-h") {
        println!(
            "cargo rsx <doctor|lint|fmt|migrate|convert|expand|preview|lsp> [path] [options]\n\
  doctor       Check resolved GPUI identities and Kit/component compatibility\n\
  lint         Diagnose RSX macros throughout the Cargo workspace\n\
  fmt          Format RSX macros; prints a file unless --write or --check\n\
  migrate      Convert px numeric classes to Tailwind units (dry run by default)\n\
  convert      Convert JSX/TSX to RSX; unsupported behavior gets placeholders\n\
  expand       Expand a standalone RSX body to Rust\n\
  preview      Preview static layout (file or Cargo project; --index selects macro)\n\
  lsp          Run the language server over stdio\n\n\
Options: --manifest-path PATH --json --strict --tailwind --deny-warnings\n\
         --check --write --output PATH --watch --port N --index N --offline\n\
Migration: enable tailwind-scale or set spacing-scale=\"tailwind\" after migrating.\n\
Formatting preserves surrounding Rust and leaves commented macro bodies unchanged."
        );
        return Ok(0);
    }
    if matches!(command.as_str(), "--version" | "-V") {
        println!("cargo-rsx {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    if command == "lsp" {
        return lsp::run().map(|()| 0);
    }
    let mut opt = Options {
        port: 9230,
        ..Options::default()
    };
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        let mut value = || {
            i += 1;
            args.get(i)
                .cloned()
                .ok_or_else(|| format!("{arg} requires a value"))
        };
        match arg.as_str() {
            "--manifest-path" => opt.manifest = Some(value()?.into()),
            "--output" => opt.output = Some(value()?.into()),
            "--index" => opt.index = value()?.parse().map_err(|_| "invalid --index")?,
            "--port" => opt.port = value()?.parse().map_err(|_| "invalid --port")?,
            "--json" => opt.json = true,
            "--strict" => opt.strict = true,
            "--tailwind" => opt.tailwind = true,
            "--deny-warnings" => opt.deny_warnings = true,
            "--check" => opt.check = true,
            "--write" => opt.write = true,
            "--watch" => opt.watch = true,
            "--offline" => opt.offline = true,
            "-" => opt.path = Some(PathBuf::from("-")),
            flag if flag.starts_with('-') => return Err(format!("unknown option {flag}")),
            _ if opt.path.is_none() => opt.path = Some(arg.into()),
            _ => return Err(format!("unexpected argument {arg}")),
        }
        i += 1;
    }
    if opt.write && opt.check {
        return Err("--write and --check are mutually exclusive".into());
    }
    match command.as_str() {
        "doctor" => {
            let report = doctor::inspect(&metadata(&opt, false)?);
            if opt.json {
                println!("{report}");
            } else {
                for engine in report["engines"].as_array().unwrap() {
                    println!(
                        "GPUI: {} {} ({})",
                        engine["name"], engine["version"], engine["source"]
                    );
                }
                for key in ["warnings", "errors"] {
                    for message in report[key].as_array().unwrap() {
                        println!("{key}: {}", message.as_str().unwrap());
                    }
                }
                println!(
                    "doctor: {}",
                    if report["ok"] == true {
                        "passed"
                    } else {
                        "failed"
                    }
                );
            }
            Ok(i32::from(report["ok"] != true))
        }
        "lint" => lint(&opt),
        "fmt" | "migrate" => rewrite(&opt, command == "migrate"),
        "expand" | "convert" => {
            let source = input(opt.path.as_deref())?;
            let result = if command == "expand" {
                gpui_rsx_tools::expand(&source, mode(&opt, opt.path.as_deref())?)?
            } else {
                serde_json::to_string_pretty(&gpui_rsx_tools::convert::convert(&source)?)
                    .map_err(|e| e.to_string())?
            };
            output(&opt, &result)?;
            Ok(0)
        }
        "preview" => preview::run(&opt),
        _ => Err(format!("unknown command {command}; run cargo rsx --help")),
    }
}

fn metadata(opt: &Options, no_deps: bool) -> Result<Value, String> {
    let mut cmd = Command::new("cargo");
    cmd.args(["metadata", "--format-version", "1"]);
    if no_deps {
        cmd.arg("--no-deps");
    }
    if opt.offline {
        cmd.arg("--offline");
    }
    if let Some(path) = &opt.manifest {
        cmd.arg("--manifest-path").arg(path);
    }
    let result = cmd
        .output()
        .map_err(|e| format!("cannot run cargo metadata: {e}"))?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).into_owned());
    }
    serde_json::from_slice(&result.stdout).map_err(|e| e.to_string())
}

fn files(opt: &Options) -> Result<BTreeSet<PathBuf>, String> {
    fn collect(path: &Path, out: &mut BTreeSet<PathBuf>) -> Result<(), String> {
        let meta =
            std::fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if meta.file_type().is_symlink() {
            return Ok(());
        }
        if meta.is_file() {
            if path.extension().is_some_and(|s| s == "rs") {
                out.insert(path.into());
            }
        } else if meta.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if matches!(
                    entry.file_name().to_str(),
                    Some("target" | "node_modules" | ".git" | "dist")
                ) {
                    continue;
                }
                collect(&entry.path(), out)?;
            }
        }
        Ok(())
    }
    let mut out = BTreeSet::new();
    if let Some(path) = &opt.path {
        collect(path, &mut out)?;
        return Ok(out);
    }
    let data = metadata(opt, true)?;
    for package in data["packages"]
        .as_array()
        .ok_or("missing Cargo packages")?
    {
        let manifest = Path::new(
            package["manifest_path"]
                .as_str()
                .ok_or("missing manifest path")?,
        );
        let root = manifest.parent().ok_or("manifest has no parent")?;
        for dir in ["src", "tests", "examples", "benches", "build.rs"] {
            let path = root.join(dir);
            if path.exists() {
                collect(&path, &mut out)?;
            }
        }
    }
    Ok(out)
}

pub(crate) fn mode(opt: &Options, path: Option<&Path>) -> Result<ClassMode, String> {
    let anchor = path.map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_owned()));
    let mut dir = anchor
        .as_deref()
        .and_then(|p| if p.is_dir() { Some(p) } else { p.parent() });
    let mut theme = None;
    while let Some(current) = dir {
        if current.join("Cargo.toml").exists() {
            theme = Theme::load_from_dir(current)?;
            break;
        }
        dir = current.parent();
    }
    if theme.is_none()
        && let Some(manifest) = &opt.manifest
    {
        theme = Theme::load_from_dir(manifest.parent().unwrap_or(Path::new(".")))?;
    }
    Ok(ClassMode {
        strict: opt.strict,
        scale: if opt.tailwind {
            SpacingScale::Tailwind
        } else {
            theme
                .as_ref()
                .and_then(|t| t.scale)
                .unwrap_or(SpacingScale::Px)
        },
        theme,
        report_dynamic_at_exit: false,
    })
}

fn is_stdin(opt: &Options) -> bool {
    opt.path.as_deref() == Some(Path::new("-"))
}

/// Every input file with its contents; `-` reads a single document from stdin.
fn sources(opt: &Options) -> Result<Vec<(PathBuf, String)>, String> {
    if is_stdin(opt) {
        return Ok(vec![(PathBuf::from("-"), input(None)?)]);
    }
    files(opt)?
        .into_iter()
        .map(|path| {
            let source =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok((path, source))
        })
        .collect()
}

/// The path to resolve `rsx.toml` from; stdin falls back to `--manifest-path`.
fn mode_path(path: &Path) -> Option<&Path> {
    (path != Path::new("-")).then_some(path)
}

fn lint(opt: &Options) -> Result<i32, String> {
    let mut reports = Vec::new();
    let mut failed = false;
    for (path, source) in sources(opt)? {
        for diagnostic in gpui_rsx_tools::analyze_rust(&source, mode(opt, mode_path(&path))?) {
            failed |= diagnostic.severity == "error"
                || (opt.deny_warnings && diagnostic.severity == "warning");
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

fn rewrite(opt: &Options, migrate: bool) -> Result<i32, String> {
    if opt.write && is_stdin(opt) {
        return Err("--write cannot be used with stdin".into());
    }
    // Rewrite everything in memory first so an error never leaves the tree half-written.
    let mut rewritten = Vec::new();
    for (path, source) in sources(opt)? {
        if migrate && mode(opt, mode_path(&path))?.scale == SpacingScale::Tailwind {
            return Err(format!(
                "{} already uses Tailwind scale; refusing a second migration",
                path.display()
            ));
        }
        let result = if migrate {
            gpui_rsx_tools::format::migrate_rust(&source)
        } else {
            gpui_rsx_tools::format::rust(&source)
        }
        .map_err(|e| format!("{}: {e}", path.display()))?;
        if result != source {
            rewritten.push((path, result));
        } else if is_stdin(opt) && !opt.check {
            // As a filter, stdin input is echoed even when already formatted.
            print!("{result}");
        }
    }
    let changed = !rewritten.is_empty();
    for (path, result) in rewritten {
        if opt.write {
            std::fs::write(&path, result).map_err(|e| format!("{}: {e}", path.display()))?;
            eprintln!("updated {}", path.display());
        } else if opt.check {
            eprintln!("would change {}", path.display());
        } else if is_stdin(opt) {
            print!("{result}");
        } else {
            println!("// {}\n{result}", path.display());
        }
    }
    if migrate {
        eprintln!(
            "After applying once, enable tailwind-scale or [options] spacing-scale = \"tailwind\". Dynamic class expressions require manual review."
        );
    }
    Ok(i32::from(opt.check && changed))
}

fn input(path: Option<&Path>) -> Result<String, String> {
    match path.filter(|p| *p != Path::new("-")) {
        Some(path) => std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display())),
        None => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|e| e.to_string())?;
            Ok(source)
        }
    }
}
fn output(opt: &Options, result: &str) -> Result<(), String> {
    if let Some(path) = &opt.output {
        std::fs::write(path, result).map_err(|e| e.to_string())?;
    } else {
        println!("{result}");
    }
    Ok(())
}
