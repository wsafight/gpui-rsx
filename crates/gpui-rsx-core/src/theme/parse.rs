//! `rsx.toml` parsing and token validation.
use super::*;

impl Theme {
    /// Parses `rsx.toml` contents.
    pub fn parse(source: &str) -> Result<Self, String> {
        let mut theme = Theme::default();
        let mut section = String::new();
        let mut preset = None;
        let mut context = "cx".to_owned();
        let mut keys = HashSet::new();

        for (index, raw_line) in source.lines().enumerate() {
            let line_no = index + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_owned();
                if !matches!(
                    section.as_str(),
                    "options"
                        | "colors"
                        | "radius"
                        | "spacing"
                        | "text-size"
                        | "line-height"
                        | "opacity"
                        | "shadow"
                ) {
                    return Err(format!(
                        "rsx.toml:{line_no}: unknown section `[{section}]` (expected options, colors, radius, spacing, text-size, line-height, opacity or shadow)"
                    ));
                }
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("rsx.toml:{line_no}: expected `key = value`"))?;
            let key = key.trim().trim_matches('"').to_owned();
            if key.is_empty()
                || !key
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(format!("rsx.toml:{line_no}: invalid key `{key}`"));
            }
            if !keys.insert((section.clone(), key.clone())) {
                return Err(format!(
                    "rsx.toml:{line_no}: duplicate key `{key}` in [{section}]"
                ));
            }
            let value =
                parse_value(value.trim()).map_err(|e| format!("rsx.toml:{line_no}: {e}"))?;

            match section.as_str() {
                "options" => {
                    let RawValue::Str(value) = value else {
                        return Err(format!("rsx.toml:{line_no}: `{key}` expects a string"));
                    };
                    match key.as_str() {
                        "spacing-scale" => {
                            theme.scale = Some(match value.as_str() {
                                "px" => SpacingScale::Px,
                                "tailwind" => SpacingScale::Tailwind,
                                other => {
                                    return Err(format!(
                                        "rsx.toml:{line_no}: spacing-scale must be \"px\" or \"tailwind\", found \"{other}\""
                                    ));
                                }
                            })
                        }
                        "preset" => preset = Some(value),
                        "context" => context = value,
                        "dynamic-runtime" => {
                            validate_runtime_path(&value)
                                .map_err(|e| format!("rsx.toml:{line_no}: {e}"))?;
                            theme.dynamic_runtime = Some(value);
                        }
                        other => {
                            return Err(format!(
                                "rsx.toml:{line_no}: unknown option `{other}` (expected spacing-scale, preset, context or dynamic-runtime)"
                            ));
                        }
                    }
                }
                "colors" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(color) => parse_color(&color).ok_or_else(|| {
                            format!(
                                "rsx.toml:{line_no}: `{key}` is not a #rgb/#rrggbb/#rrggbbaa color"
                            )
                        })?,
                    };
                    theme.colors.insert(key, token);
                }
                "radius" | "spacing" | "text-size" | "line-height" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(length) => parse_length(&length).ok_or_else(|| {
                            format!("rsx.toml:{line_no}: `{key}` must be a length like \"12px\" or \"0.75rem\"")
                        })?,
                    };
                    let map = match section.as_str() {
                        "radius" => &mut theme.radius,
                        "spacing" => &mut theme.spacing,
                        "text-size" => &mut theme.text_size,
                        "line-height" => &mut theme.line_height,
                        _ => unreachable!("section was checked above"),
                    };
                    map.insert(key, token);
                }
                "opacity" => {
                    let token = match value {
                        RawValue::Expr(expr) => TokenValue::Expr(validate_expr(&expr, line_no)?),
                        RawValue::Str(opacity) => parse_opacity(&opacity).ok_or_else(|| {
                            format!(
                                "rsx.toml:{line_no}: `{key}` must be an opacity between 0 and 1, such as \"0.6\" or \"60%\""
                            )
                        })?,
                    };
                    theme.opacity.insert(key, token);
                }
                "shadow" => {
                    let RawValue::Expr(expr) = value else {
                        return Err(format!(
                            "rsx.toml:{line_no}: `{key}` expects an inline Rust expression like {{ expr = \"cx.theme().shadow\" }}"
                        ));
                    };
                    theme
                        .shadow
                        .insert(key, TokenValue::Expr(validate_expr(&expr, line_no)?));
                }
                _ => {
                    return Err(format!(
                        "rsx.toml:{line_no}: `{key}` must be inside a section"
                    ));
                }
            }
        }

        match preset.as_deref() {
            None => {}
            Some("gpui-component") => {
                validate_expr(&context, 0)?;
                for field in GPUI_COMPONENT_THEME_COLORS {
                    let name = field.replace('_', "-");
                    theme.colors.entry(name).or_insert_with(|| {
                        TokenValue::Expr(format!("{context}.theme().colors.{field}"))
                    });
                }
            }
            Some(other) => {
                return Err(format!(
                    "rsx.toml: unknown preset \"{other}\" (supported: \"gpui-component\")"
                ));
            }
        }

        theme.validate()?;
        Ok(theme)
    }

    /// Validates in-memory token types, names and expressions before expansion.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(path) = self.dynamic_runtime() {
            validate_runtime_path(path)?;
        }
        for token in self.tokens() {
            if token.name.is_empty()
                || !token
                    .name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return Err(format!("rsx.toml: invalid token name `{}`", token.name));
            }
            let valid = match &token.value {
                TokenValue::Expr(expr) => syn::parse_str::<syn::Expr>(expr).is_ok(),
                TokenValue::Color { hex, alpha } => {
                    token.kind == ThemeTokenKind::Color && (*alpha || *hex <= 0xffffff)
                }
                TokenValue::Length(expr) => {
                    matches!(
                        token.kind,
                        ThemeTokenKind::Radius
                            | ThemeTokenKind::Spacing
                            | ThemeTokenKind::TextSize
                            | ThemeTokenKind::LineHeight
                    ) && syn::parse_str::<syn::Expr>(expr).is_ok()
                }
                TokenValue::Number(value) => {
                    token.kind == ThemeTokenKind::Opacity
                        && value.is_finite()
                        && (0.0..=1.0).contains(value)
                }
            };
            if !valid {
                return Err(format!(
                    "rsx.toml: invalid value for [{}] token `{}`",
                    token.kind.as_str(),
                    token.name
                ));
            }
        }
        if self
            .text_size
            .keys()
            .any(|name| self.colors.contains_key(&format!("size-{name}")))
        {
            return Err(
                "rsx.toml: a text-size class conflicts with a color token named `size-*`".into(),
            );
        }
        Ok(())
    }
}

fn validate_runtime_path(value: &str) -> Result<(), String> {
    let valid = syn::parse_str::<syn::Path>(value).is_ok_and(|path| {
        path.segments
            .iter()
            .all(|segment| matches!(segment.arguments, syn::PathArguments::None))
    });
    if valid {
        Ok(())
    } else {
        Err("dynamic-runtime must be a Rust module path such as \"rsx_runtime\" or \"crate::rsx_runtime\"".into())
    }
}

enum RawValue {
    Str(String),
    Expr(String),
}

fn parse_value(value: &str) -> Result<RawValue, String> {
    if let Some(inner) = value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
        let (key, rest) = inner
            .split_once('=')
            .ok_or("inline tables must look like { expr = \"...\" }")?;
        if key.trim() != "expr" {
            return Err(format!(
                "unknown inline key `{}` (expected `expr`)",
                key.trim()
            ));
        }
        return parse_string(rest.trim()).map(RawValue::Expr);
    }
    parse_string(value).map(RawValue::Str)
}

fn parse_string(value: &str) -> Result<String, String> {
    let inner = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .ok_or_else(|| format!("expected a quoted string, found `{value}`"))?;
    Ok(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, c) in line.char_indices() {
        match c {
            // `\"` inside a string (which `parse_string` unescapes) does not close it.
            _ if escaped => escaped = false,
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn validate_expr(expr: &str, line_no: usize) -> Result<String, String> {
    syn::parse_str::<syn::Expr>(expr)
        .map(|_| expr.to_owned())
        .map_err(|e| format!("rsx.toml:{line_no}: `{expr}` is not a Rust expression: {e}"))
}

fn parse_color(color: &str) -> Option<TokenValue> {
    let hex = color
        .strip_prefix('#')
        .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))?;
    let digits = |s: &str| u32::from_str_radix(s, 16).ok();
    match hex.len() {
        6 => Some(TokenValue::Color {
            hex: digits(hex)?,
            alpha: false,
        }),
        8 => Some(TokenValue::Color {
            hex: digits(hex)?,
            alpha: true,
        }),
        3 => {
            let v = digits(hex)?;
            let (r, g, b) = ((v >> 8) & 0xf, (v >> 4) & 0xf, v & 0xf);
            Some(TokenValue::Color {
                hex: (r * 0x11) << 16 | (g * 0x11) << 8 | (b * 0x11),
                alpha: false,
            })
        }
        _ => None,
    }
}

fn parse_length(length: &str) -> Option<TokenValue> {
    let number = |raw: &str| raw.parse::<f32>().ok().filter(|n| n.is_finite());
    if let Some(raw) = length.strip_suffix("px") {
        return Some(TokenValue::Length(format!("px({:?})", number(raw)?)));
    }
    if let Some(raw) = length.strip_suffix("rem") {
        return Some(TokenValue::Length(format!("rems({:?})", number(raw)?)));
    }
    None
}

fn parse_opacity(opacity: &str) -> Option<TokenValue> {
    let value = if let Some(raw) = opacity.strip_suffix('%') {
        raw.parse::<f32>().ok()? / 100.0
    } else {
        opacity.parse::<f32>().ok()?
    };
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (0.0..=1.0).contains(value))
        .map(TokenValue::Number)
}
