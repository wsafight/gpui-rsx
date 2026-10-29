//! Repository automation. Run `cargo xtask help`.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use xtask::{gen_tables, gpui_api};

const HELP: &str = "\
cargo xtask <command>

Commands:
  gen-tables [--gpui-src <dir>] [--check]
      Regenerate tables, API fixtures, the doctest GPUI stub and the demo API contract
      from GPUI sources. <dir> is a package source root; defaults to $GPUI_SOURCE_ROOT or the
      package sources resolved through `cargo metadata` on demo/Cargo.toml.
      --check fails instead of writing when any output is stale.
  spec [--check]
      Regenerate spec/rsx-spec.json (the LLM syntax package).
  compat-matrix [--check]
      Rewrite the verified-version blocks in README/docs from demo/Cargo.lock.
  bump <version> | bump --check
      Set the release version in manifests, workspace lockfile entries, editor packages and
      install snippets, then refresh the compatibility matrix. --check verifies that every
      site already matches [workspace.package].version.
  line-limit [--max <lines>]
      Fail when a hand-written Rust file has more than <lines> (default 600) non-blank
      lines outside #[cfg(test)] items. Tests, benches and generated files are skipped.
  check-gpui\n      Require one matching published GPUI snapshot across Kit, base and components.\n      Print the verified package versions as JSON.\n  bench-check [--threshold <fraction>] [--criterion-dir <dir>]
      Fail when any criterion benchmark regressed more than <fraction> (default 0.10)
      against the baseline it was compared with (`cargo bench -- --baseline <name>`).
      Uses the lower confidence bound to avoid failing on measurement noise.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("gen-tables") => gen_tables_cmd(&args[1..]),
        Some("spec") => {
            check_flag(&args[1..]).and_then(|check| xtask::spec::run(&workspace_root(), check))
        }
        Some("compat-matrix") => {
            check_flag(&args[1..]).and_then(|check| xtask::compat::run(&workspace_root(), check))
        }
        Some("check-gpui") if args.len() == 1 => gpui_api::demo_metadata(&workspace_root(), true)
            .and_then(|data| xtask::upstream::validate_graph(&data))
            .map(|versions| println!("{versions}")),
        Some("bump") => match &args[1..] {
            [flag] if flag == "--check" => xtask::bump::run(&workspace_root(), None, true),
            [version] if !version.starts_with('-') => {
                xtask::bump::run(&workspace_root(), Some(version), false)
            }
            _ => Err("expected `bump <version>` or `bump --check`".into()),
        },
        Some("line-limit") => line_limit_cmd(&args[1..])
            .and_then(|max| xtask::line_limit::run(&workspace_root(), max)),
        Some("bench-check") => xtask::bench::run(&workspace_root(), &args[1..]),
        Some("help") | Some("-h") | Some("--help") | None => {
            print!("{HELP}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{HELP}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn check_flag(args: &[String]) -> Result<bool, String> {
    match args {
        [] => Ok(false),
        [flag] if flag == "--check" => Ok(true),
        _ => Err("expected no options or --check".into()),
    }
}

fn line_limit_cmd(args: &[String]) -> Result<usize, String> {
    match args {
        [] => Ok(xtask::line_limit::DEFAULT_MAX),
        [flag, max] if flag == "--max" => max.parse().map_err(|_| format!("invalid --max {max}")),
        _ => Err("expected no options or --max <lines>".into()),
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace root")
        .to_path_buf()
}

fn gen_tables_cmd(args: &[String]) -> Result<(), String> {
    let mut gpui_src = None;
    let mut check = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--gpui-src" => {
                gpui_src = Some(PathBuf::from(
                    iter.next().ok_or("--gpui-src needs a value")?,
                ))
            }
            "--check" => check = true,
            other => return Err(format!("unknown gen-tables option `{other}`")),
        }
    }

    let root = workspace_root();
    let (gpui_root, macros_root) = gpui_api::locate_sources(gpui_src, &root)?;
    eprintln!("reading GPUI sources from {}", gpui_root.display());
    let api = gpui_api::GpuiApi::load(&gpui_root, &macros_root)?;
    let component_root = match std::env::var_os("GPUI_COMPONENT_SOURCE_ROOT") {
        Some(dir) => PathBuf::from(dir),
        None => gpui_api::metadata_package_root(&root, "gpui-component", true)?,
    };
    let theme_fields = gen_tables::theme_color_fields(&component_root)?;
    let revs = gen_tables::locked_revisions(&root)?;
    let outputs = gen_tables::generate(&api, &theme_fields, &revs, &root)?;
    xtask::write_outputs(&outputs.files, check)
}
