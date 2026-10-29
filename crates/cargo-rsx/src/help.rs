//! Command help uses the same option allowlist as argument validation.
pub(super) fn print(command: &str, operation: Option<&str>) -> Result<(), String> {
    if command == "theme" {
        let operation = operation
            .filter(|s| matches!(*s, "check" | "list" | "explain"))
            .unwrap_or("<check|list|explain>");
        println!(
            "cargo rsx theme {operation} {}[--manifest-path PATH] [--json] [--offline]\nInspect rsx.toml tokens without compiling GPUI.",
            if operation == "explain" { "CLASS " } else { "" }
        );
        return Ok(());
    }
    if command == "lsp" {
        println!(
            "cargo rsx lsp\nRun the RSX language server over stdio. Formatting is opt-in through initializationOptions.formatting."
        );
        return Ok(());
    }
    let allowed =
        super::allowed_options(command).ok_or_else(|| format!("unknown command {command}"))?;
    println!("cargo rsx {command} [path|-] [options]");
    for &option in allowed {
        let help = match option {
            "--manifest-path" => "PATH  Select Cargo.toml",
            "--output" => "PATH  Write the result to a file",
            "--json" => "Emit structured JSON",
            "--strict" => "Treat unsupported static classes as errors",
            "--tailwind" => "Use Tailwind's 4px scale; convert emits rsx_tw!",
            "--deny-warnings" => "Fail on warnings even if output is filtered",
            "--min-severity" => {
                "error|warning|information  Filter diagnostics (default: information)"
            }
            "--quiet" => "Show only errors (alias: -q)",
            "--check" => "Fail if formatting or migration would change files",
            "--write" => "Apply formatting or migration to files",
            "--watch" => "Refresh the preview when source files change",
            "--port" => "N  Preview HTTP port (default: 9230)",
            "--index" => "N  Select the macro to preview (default: 0)",
            "--offline" => "Run Cargo metadata without network access",
            _ => "",
        };
        println!("  {option} {help}");
    }
    if command == "lint" {
        println!(
            "Stdin accepts a Rust file or a standalone RSX body. Filtering never changes the exit status."
        );
    }
    Ok(())
}
