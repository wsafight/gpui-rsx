mod config;
mod doctor;
mod help;
mod lint;
mod lsp;
mod preview;
mod theme;

use gpui_rsx_core::{ClassMode, SpacingScale};
use serde_json::Value;
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
    min_severity: Option<String>,
    quiet: bool,
    check: bool,
    write: bool,
    watch: bool,
    offline: bool,
    index: usize,
    port: u16,
}

impl Options {
    fn mode_options(&self) -> config::ModeOptions<'_> {
        config::ModeOptions {
            strict: self.strict,
            tailwind: self.tailwind,
            manifest: self.manifest.as_deref(),
            offline: self.offline,
        }
    }
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
            "cargo rsx <doctor|lint|fmt|migrate|convert|expand|preview|theme|lsp> [path] [options]\n\
  doctor       Check resolved GPUI identities and Kit/component compatibility\n\
  lint         Diagnose RSX macros throughout the Cargo workspace\n\
  fmt          Format RSX macros; prints a file unless --write or --check\n\
  migrate      Convert px numeric classes to Tailwind units (dry run by default)\n\
  convert      Convert JSX/TSX to RSX; unsupported behavior gets placeholders\n\
  expand       Expand a standalone RSX body to Rust\n\
  preview      Preview static layout (file or Cargo project; --index selects macro)\n\
  theme        Check, list or explain rsx.toml theme tokens\n\
  lsp          Run the language server over stdio\n\n\
Options: --manifest-path PATH --json --strict --tailwind --deny-warnings\n\
         --check --write --output PATH --watch --port N --index N --offline\n\
         lint: --min-severity error|warning|information --quiet\n\
Migration: enable tailwind-scale or set spacing-scale=\"tailwind\" after migrating.\n\
Formatting preserves surrounding Rust and leaves commented macro bodies unchanged."
        );
        return Ok(0);
    }
    if matches!(command.as_str(), "--version" | "-V") {
        println!("cargo-rsx {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    if args.iter().skip(1).any(|a| a == "--help" || a == "-h") {
        help::print(&command, args.get(1).map(String::as_str))?;
        return Ok(0);
    }
    if command == "theme" {
        return theme::run(&args[1..]);
    }
    if command == "lsp" {
        if args.len() > 1 {
            return Err("lsp takes no command-line options; run cargo rsx lsp --help".into());
        }
        return lsp::run().map(|()| 0);
    }
    let mut opt = Options {
        port: 9230,
        ..Options::default()
    };
    let mut flags = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with("--") {
            flags.push(arg.clone());
        }
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
            "--min-severity" => {
                let level = value()?;
                if !matches!(level.as_str(), "error" | "warning" | "information") {
                    return Err("--min-severity expects error, warning or information".into());
                }
                opt.min_severity = Some(level);
            }
            "--quiet" | "-q" => {
                opt.quiet = true;
                if arg == "-q" {
                    flags.push("--quiet".into());
                }
            }
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
    if let Some(allowed) = allowed_options(&command)
        && let Some(flag) = flags.iter().find(|flag| !allowed.contains(&flag.as_str()))
    {
        return Err(format!("{flag} does not apply to `cargo rsx {command}`"));
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
                    let field = |key: &str| engine[key].as_str().unwrap_or("unknown").to_owned();
                    println!(
                        "GPUI: {} {} ({})",
                        field("name"),
                        field("version"),
                        field("source")
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
        "lint" => lint::run(&opt),
        "fmt" | "migrate" => rewrite(&opt, command == "migrate"),
        "expand" | "convert" => {
            let source = input(opt.path.as_deref())?;
            let result = if command == "expand" {
                gpui_rsx_tools::expand(
                    &source,
                    mode(&opt, opt.path.as_deref().and_then(mode_path))?,
                )?
            } else {
                serde_json::to_string_pretty(&gpui_rsx_tools::convert::convert_with_options(
                    &source,
                    {
                        let mut options = gpui_rsx_tools::convert::ConversionOptions::default();
                        options.tailwind = opt.tailwind;
                        options
                    },
                )?)
                .map_err(|e| e.to_string())?
            };
            output(&opt, &result)?;
            Ok(0)
        }
        "preview" => preview::run(&opt),
        _ => Err(format!("unknown command {command}; run cargo rsx --help")),
    }
}

/// Options each command reads; anything else would be silently ignored, so it is rejected.
fn allowed_options(command: &str) -> Option<&'static [&'static str]> {
    Some(match command {
        "doctor" => &["--manifest-path", "--json", "--offline"],
        "lint" => &[
            "--manifest-path",
            "--json",
            "--strict",
            "--tailwind",
            "--deny-warnings",
            "--min-severity",
            "--quiet",
            "--offline",
        ],
        "fmt" => &["--manifest-path", "--check", "--write", "--offline"],
        "migrate" => &[
            "--manifest-path",
            "--check",
            "--write",
            "--tailwind",
            "--offline",
        ],
        "expand" => &[
            "--manifest-path",
            "--strict",
            "--tailwind",
            "--output",
            "--offline",
        ],
        "convert" => &["--output", "--tailwind"],
        "preview" => &[
            "--manifest-path",
            "--strict",
            "--tailwind",
            "--watch",
            "--port",
            "--index",
            "--output",
            "--offline",
        ],
        _ => return None,
    })
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
        // A file named explicitly is checked whatever its extension; only directory walks
        // filter for `.rs`.
        if path.is_file() {
            out.insert(path.clone());
            return Ok(out);
        }
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

fn mode(opt: &Options, path: Option<&Path>) -> Result<ClassMode, String> {
    config::mode(opt.mode_options(), path)
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

fn rewrite(opt: &Options, migrate: bool) -> Result<i32, String> {
    if opt.write && is_stdin(opt) {
        return Err("--write cannot be used with stdin".into());
    }
    // Rewrite everything in memory first so an error never leaves the tree half-written.
    let mut rewritten = Vec::new();
    for (path, source) in sources(opt)? {
        if migrate && mode(opt, mode_path(&path))?.scale() == SpacingScale::Tailwind {
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
