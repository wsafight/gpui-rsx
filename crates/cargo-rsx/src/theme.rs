//! Theme inspection commands share the macro's configuration parser.
use super::{Options, resolve_theme};
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

    let theme = match resolve_theme(&opt, None) {
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
