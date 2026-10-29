//! Theme inspection commands share the macro's configuration parser.
use super::{Options, config::resolve_theme};
use gpui_rsx_core::{Theme, TokenValue};
use serde_json::{Value, json};

fn theme_value(value: &TokenValue) -> Value {
    match value {
        TokenValue::Color { hex, alpha } => {
            if *alpha {
                json!(format!("#{hex:08x}"))
            } else {
                json!(format!("#{hex:06x}"))
            }
        }
        TokenValue::Length(expr) | TokenValue::Expr(expr) => json!(expr),
        TokenValue::Number(number) => json!(number),
        _ => Value::Null,
    }
}

fn theme_listing(theme: &Theme) -> Vec<Value> {
    theme
        .class_names()
        .into_iter()
        .filter_map(|class| {
            theme.explain_class(&class).map(|explanation| {
                json!({
                    "class": explanation.class,
                    "section": explanation.token.kind.as_str(),
                    "name": explanation.token.name,
                    "value": theme_value(&explanation.token.value),
                    "expansion": explanation.expansion,
                })
            })
        })
        .collect()
}

fn print_theme_error(opt: &Options, error: String) -> Result<i32, String> {
    if opt.json {
        println!("{}", json!({"ok": false, "error": error}));
    } else {
        eprintln!("theme: {error}");
    }
    Ok(1)
}

pub(super) fn run(args: &[String]) -> Result<i32, String> {
    let operation = args.first().map(String::as_str).unwrap_or("check");
    if matches!(operation, "--help" | "-h" | "help") {
        println!("cargo rsx theme <check|list|explain CLASS> [--manifest-path PATH] [--json]");
        return Ok(0);
    }
    if !matches!(operation, "check" | "list" | "explain") {
        return Err(format!(
            "unknown theme command {operation}; expected check, list or explain"
        ));
    }
    let mut opt = Options::default();
    let mut class = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest-path" => {
                i += 1;
                opt.manifest = Some(
                    args.get(i)
                        .ok_or("--manifest-path requires a value")?
                        .into(),
                );
            }
            "--json" => opt.json = true,
            "--offline" => opt.offline = true,
            value if value.starts_with('-') => return Err(format!("unknown option {value}")),
            value if class.is_none() => class = Some(value.to_owned()),
            value => return Err(format!("unexpected argument {value}")),
        }
        i += 1;
    }
    if operation != "explain" && class.is_some() {
        return Err(format!(
            "theme {operation} does not accept a class argument"
        ));
    }
    if operation == "explain" && class.is_none() {
        return Err("theme explain requires a class such as bg-primary".into());
    }

    let theme = match resolve_theme(opt.manifest.as_deref(), None) {
        Ok(theme) => theme,
        Err(error) => return print_theme_error(&opt, error),
    };
    let Some(theme) = theme else {
        if opt.json {
            println!(
                "{}",
                json!({"ok": operation == "check", "present": false, "tokens": []})
            );
        } else {
            println!("no rsx.toml found");
        }
        return Ok(i32::from(operation != "check"));
    };

    match operation {
        "check" => {
            if opt.json {
                println!(
                    "{}",
                    json!({"ok": true, "present": true, "path": theme.path().map(|p| p.display().to_string()), "tokenCount": theme.tokens().len()})
                );
            } else {
                println!(
                    "valid {} ({} tokens)",
                    theme
                        .path()
                        .map_or_else(|| "in-memory theme".into(), |p| p.display().to_string()),
                    theme.tokens().len()
                );
            }
            Ok(0)
        }
        "list" => {
            let entries = theme_listing(&theme);
            if opt.json {
                println!("{}", json!({"ok": true, "tokens": entries}));
            } else {
                for entry in entries {
                    println!(
                        "{} [{}:{}] -> {}",
                        entry["class"].as_str().unwrap_or(""),
                        entry["section"].as_str().unwrap_or(""),
                        entry["name"].as_str().unwrap_or(""),
                        entry["expansion"].as_str().unwrap_or("")
                    );
                }
            }
            Ok(0)
        }
        "explain" => {
            let class = class.expect("checked above");
            let Some(explanation) = theme.explain_class(&class) else {
                return print_theme_error(&opt, format!("unknown theme class `{class}`"));
            };
            let result = json!({
                "ok": true,
                "class": explanation.class,
                "section": explanation.token.kind.as_str(),
                "name": explanation.token.name,
                "value": theme_value(&explanation.token.value),
                "expansion": explanation.expansion,
            });
            if opt.json {
                println!("{result}");
            } else {
                println!(
                    "{} [{}:{}] = {}\n{}",
                    result["class"].as_str().unwrap_or(""),
                    result["section"].as_str().unwrap_or(""),
                    result["name"].as_str().unwrap_or(""),
                    result["value"],
                    result["expansion"].as_str().unwrap_or("")
                );
            }
            Ok(0)
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn project(name: &str, config: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("rsx-theme-command-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"theme-command\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        if !config.is_empty() {
            std::fs::write(dir.join("rsx.toml"), config).unwrap();
        }
        dir.join("Cargo.toml")
    }

    #[test]
    fn token_values_keep_colors_lengths_expressions_and_numbers() {
        assert_eq!(
            theme_value(&TokenValue::Color {
                hex: 0x123456,
                alpha: false
            }),
            json!("#123456")
        );
        assert_eq!(
            theme_value(&TokenValue::Color {
                hex: 0x12345678,
                alpha: true
            }),
            json!("#12345678")
        );
        assert_eq!(
            theme_value(&TokenValue::Length("px(12.0)".into())),
            json!("px(12.0)")
        );
        assert_eq!(
            theme_value(&TokenValue::Expr("cx.theme().shadow".into())),
            json!("cx.theme().shadow")
        );
        assert_eq!(theme_value(&TokenValue::Number(0.5)), json!(0.5));
    }

    #[test]
    fn listings_cover_every_token_section() {
        let theme = Theme::builder()
            .color(
                "brand",
                TokenValue::Color {
                    hex: 0x123456,
                    alpha: false,
                },
            )
            .radius("card", TokenValue::Length("px(12.0)".into()))
            .spacing("gutter", TokenValue::Length("px(16.0)".into()))
            .text_size("body", TokenValue::Length("px(14.0)".into()))
            .line_height("body", TokenValue::Length("px(20.0)".into()))
            .opacity("muted", 0.6)
            .shadow("card", "cx.theme().shadow")
            .build();
        let listing = theme_listing(&theme);
        let find = |class: &str| {
            listing
                .iter()
                .find(|entry| entry["class"] == class)
                .unwrap_or_else(|| panic!("missing {class}"))
        };
        assert_eq!(find("bg-brand")["section"], "colors");
        assert_eq!(find("bg-brand")["value"], "#123456");
        assert_eq!(find("rounded-card")["section"], "radius");
        assert_eq!(find("p-gutter")["section"], "spacing");
        assert_eq!(find("text-size-body")["section"], "text-size");
        assert_eq!(find("leading-body")["section"], "line-height");
        assert_eq!(find("opacity-muted")["value"].as_f64(), Some(0.6f32 as f64));
        assert_eq!(find("shadow-card")["value"], "cx.theme().shadow");
    }

    #[test]
    fn help_and_unknown_operations_report_usage() {
        assert_eq!(run(&arguments(&["--help"])), Ok(0));
        assert_eq!(run(&arguments(&["help"])), Ok(0));
        let error = run(&arguments(&["nope"])).unwrap_err();
        assert!(error.contains("unknown theme command"), "{error}");
        let error = run(&arguments(&["explain"])).unwrap_err();
        assert!(error.contains("requires a class"), "{error}");
        let error = run(&arguments(&["list", "bg-brand"])).unwrap_err();
        assert!(error.contains("does not accept a class"), "{error}");
        let error = run(&arguments(&["check", "--unknown"])).unwrap_err();
        assert!(error.contains("unknown option"), "{error}");
        let error = run(&arguments(&["check", "--manifest-path"])).unwrap_err();
        assert!(error.contains("requires a value"), "{error}");
        let error = run(&arguments(&["check", "extra", "more"])).unwrap_err();
        assert!(error.contains("unexpected argument"), "{error}");
    }

    #[test]
    fn check_list_and_explain_read_the_nearest_rsx_toml() {
        let manifest = project(
            "present",
            "[colors]\nbrand = \"#123456\"\n[opacity]\nmuted = \"60%\"\n",
        );
        let manifest = manifest.to_str().unwrap();
        assert_eq!(
            run(&arguments(&["check", "--manifest-path", manifest])),
            Ok(0)
        );
        assert_eq!(
            run(&arguments(&["list", "--manifest-path", manifest])),
            Ok(0)
        );
        assert_eq!(
            run(&arguments(&["list", "--json", "--manifest-path", manifest])),
            Ok(0)
        );
        assert_eq!(
            run(&arguments(&[
                "explain",
                "bg-brand",
                "--manifest-path",
                manifest
            ])),
            Ok(0)
        );
        assert_eq!(
            run(&arguments(&[
                "explain",
                "hover:opacity-muted",
                "--json",
                "--manifest-path",
                manifest
            ])),
            Ok(0)
        );
        // An unknown class is a failure even though the theme loaded.
        assert_eq!(
            run(&arguments(&[
                "explain",
                "bg-missing",
                "--manifest-path",
                manifest
            ])),
            Ok(1)
        );
        std::fs::remove_dir_all(std::path::Path::new(manifest).parent().unwrap()).unwrap();
    }

    #[test]
    fn missing_config_files_are_reported_without_failing_check() {
        let manifest = project("absent", "");
        let manifest = manifest.to_str().unwrap();
        assert_eq!(
            run(&arguments(&["check", "--manifest-path", manifest])),
            Ok(0)
        );
        assert_eq!(
            run(&arguments(&[
                "check",
                "--json",
                "--manifest-path",
                manifest
            ])),
            Ok(0)
        );
        assert_eq!(
            run(&arguments(&["list", "--manifest-path", manifest])),
            Ok(1)
        );
        std::fs::remove_dir_all(std::path::Path::new(manifest).parent().unwrap()).unwrap();
    }
}
