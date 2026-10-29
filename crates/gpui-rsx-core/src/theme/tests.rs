use super::cache::{MAX_THEME_CACHE_ENTRIES, ThemeCache};
use super::*;

#[test]
fn parses_tokens_and_options() {
    let theme = Theme::parse(
        r##"
        # design tokens
        [options]
        spacing-scale = "tailwind"

        [colors]
        primary = "#2563eb"
        overlay = "#00000080"   # alpha
        surface = { expr = "cx.theme().background" }

        [radius]
        card = "12px"

        [spacing]
        gutter = "1.5rem"
        "##,
    )
    .unwrap();
    assert_eq!(theme.scale, Some(SpacingScale::Tailwind));
    assert_eq!(
        theme.expand_class("bg-primary").unwrap().to_string(),
        quote! { .bg(rgb(2450411u32)) }.to_string()
    );
    assert!(
        theme
            .expand_class("text-overlay")
            .unwrap()
            .to_string()
            .contains("rgba")
    );
    assert_eq!(
        theme.expand_class("border-surface").unwrap().to_string(),
        quote! { .border_color(cx.theme().background) }.to_string()
    );
    assert_eq!(
        theme.expand_class("rounded-card").unwrap().to_string(),
        quote! { .rounded(px(12.0)) }.to_string()
    );
    assert_eq!(
        theme.expand_class("gap-x-gutter").unwrap().to_string(),
        quote! { .gap_x(rems(1.5)) }.to_string()
    );
    assert!(theme.expand_class("bg-unknown").is_none());
}

#[test]
fn parses_extended_token_sections_and_explains_classes() {
    let theme = Theme::parse(
        "[text-size]\nbody = \"14px\"\n[line-height]\nbody = \"20px\"\n[opacity]\nmuted = \"60%\"\n[shadow]\ncard = { expr = \"cx.theme().shadow\" }\n",
    )
    .unwrap();
    assert_eq!(theme.text_size().len(), 1);
    assert_eq!(theme.line_height().len(), 1);
    assert_eq!(theme.opacity().get("muted"), Some(&TokenValue::Number(0.6)));
    assert_eq!(theme.shadow().len(), 1);
    for class in [
        "text-size-body",
        "leading-body",
        "opacity-muted",
        "shadow-card",
    ] {
        assert!(theme.expand_class(class).is_some(), "{class}");
        assert!(theme.class_names().contains(&class.to_owned()), "{class}");
    }
    let explanation = theme.explain_class("hover:opacity-muted").unwrap();
    assert_eq!(explanation.token.kind, ThemeTokenKind::Opacity);
    assert_eq!(explanation.token.name, "muted");
    assert!(explanation.expansion.contains("opacity"));
    assert_eq!(theme.tokens().len(), 4);
    assert!(theme.explain_class("dark:opacity-muted").is_none());
    assert!(explanation.expansion.contains("hover"));
}

#[test]
fn validates_in_memory_themes_and_rejects_ambiguous_configs() {
    assert!(
        Theme::builder()
            .opacity("muted", f32::NAN)
            .try_build()
            .is_err()
    );
    assert!(Theme::builder().shadow("card", "cx.(").try_build().is_err());
    assert!(Theme::builder().opacity("muted", 0.6).try_build().is_ok());
    assert!(
        Theme::parse("[colors]\nbrand=\"#123456\"\nbrand=\"#654321\"\n")
            .unwrap_err()
            .contains("duplicate key")
    );
    assert!(
        Theme::parse("[colors]\nsize-body=\"#123456\"\n[text-size]\nbody=\"14px\"\n")
            .unwrap_err()
            .contains("conflicts")
    );
}

#[test]
fn cache_evicts_the_least_recently_used_entry() {
    let mut cache = ThemeCache::default();
    for i in 0..MAX_THEME_CACHE_ENTRIES {
        cache.insert(PathBuf::from(i.to_string()), 0, Arc::new(Theme::new()));
    }
    assert!(cache.get(Path::new("0"), 0).is_some());
    cache.insert(PathBuf::from("new"), 0, Arc::new(Theme::new()));
    assert_eq!(cache.entries.len(), MAX_THEME_CACHE_ENTRIES);
    assert!(cache.get(Path::new("1"), 0).is_none());
    assert!(cache.get(Path::new("0"), 0).is_some());
}

#[test]
fn gpui_component_preset_maps_semantic_colors() {
    let theme =
        Theme::parse("[options]\npreset = \"gpui-component\"\ncontext = \"app\"\n").unwrap();
    assert_eq!(
        theme
            .expand_class("bg-primary-foreground")
            .unwrap()
            .to_string(),
        quote! { .bg(app.theme().colors.primary_foreground) }.to_string()
    );
    // Theme.list is ListSettings; the color must bypass Theme's own fields.
    assert_eq!(
        theme.expand_class("bg-list").unwrap().to_string(),
        quote! { .bg(app.theme().colors.list) }.to_string()
    );
    assert!(
        theme
            .class_names()
            .contains(&"text-muted-foreground".to_owned())
    );
}

#[test]
fn reports_errors_with_line_numbers() {
    let err = Theme::parse("[colors]\nprimary = \"blue\"\n").unwrap_err();
    assert!(err.starts_with("rsx.toml:2:"), "{err}");
    let err = Theme::parse("[colors]\nx = { expr = \"cx.(\" }\n").unwrap_err();
    assert!(err.contains("not a Rust expression"), "{err}");
    let err = Theme::parse("[themes]\n").unwrap_err();
    assert!(err.contains("unknown section"), "{err}");
}

#[test]
fn escaped_quotes_do_not_end_strings_before_comments() {
    let theme = Theme::parse(
        r##"
        [colors]
        brand = { expr = "f(\"#a\")" } # trailing comment
        "##,
    )
    .unwrap();
    assert!(matches!(
        theme.colors.get("brand"),
        Some(TokenValue::Expr(expr)) if expr == r##"f("#a")"##
    ));
}

#[test]
fn class_names_cover_every_length_prefix() {
    let theme = Theme::parse("[spacing]\ngutter = \"12px\"\n").unwrap();
    let names = theme.class_names();
    for class in ["pt-gutter", "min-w-gutter", "gap-x-gutter", "p-gutter"] {
        assert!(names.iter().any(|n| n == class), "{class}");
        assert!(theme.expand_class(class).is_some(), "{class}");
    }
}

#[test]
fn class_names_are_derived_from_the_length_prefix_table() {
    let theme = Theme::parse("[spacing]\ngutter = \"12px\"\n").unwrap();
    let names = theme.class_names();
    for spec in LENGTH_CLASS_SPECS {
        let class = format!("{}{}", spec.prefix, "gutter");
        assert!(names.contains(&class), "{class} is missing from completion");
        assert!(
            theme.expand_class(&class).is_some(),
            "{class} does not expand"
        );
    }
    assert_eq!(
        names.iter().filter(|name| name.contains("gutter")).count(),
        LENGTH_CLASS_SPECS.len()
    );
}

#[test]
fn color_values_require_only_hex_digits() {
    for color in ["#+12345", "#12345g", "#1234567", "#"] {
        let err = Theme::parse(&format!("[colors]\nbrand = \"{color}\"\n")).unwrap_err();
        assert!(
            err.contains("not a #rgb/#rrggbb/#rrggbbaa color"),
            "{color}: {err}"
        );
    }
    // All accepted spellings still parse, and alpha is tracked.
    let theme = Theme::parse(
        "[colors]\nshort = \"#abc\"\nplain = \"#123456\"\nwith-alpha = \"#12345678\"\n",
    )
    .unwrap();
    assert!(matches!(
        theme.colors.get("short"),
        Some(TokenValue::Color {
            hex: 0xaabbcc,
            alpha: false
        })
    ));
    assert!(matches!(
        theme.colors.get("with-alpha"),
        Some(TokenValue::Color {
            hex: 0x12345678,
            alpha: true
        })
    ));
    assert!(matches!(
        theme.colors.get("plain"),
        Some(TokenValue::Color {
            hex: 0x123456,
            alpha: false
        })
    ));
}

#[test]
fn comments_after_escaped_backslashes_and_hashes_parse() {
    // A `\` inside a string escapes the next quote, so `#` after it is still a comment.
    let theme = Theme::parse(
        r##"
        [colors]
        first = { expr = "f(\"x\")" } # comment
        second = { expr = "g(\"#y\")" } # another
        "##,
    )
    .unwrap();
    assert_eq!(theme.colors.len(), 2, "{:?}", theme.colors);
    assert!(matches!(
        theme.colors.get("second"),
        Some(TokenValue::Expr(expr)) if expr == r##"g("#y")"##
    ));

    // A `#` inside a string is not a comment.
    let theme = Theme::parse("[colors]\nbrand = \"#123456\" # real comment\n").unwrap();
    assert_eq!(theme.colors.len(), 1);
}

#[test]
fn rejects_malformed_lines_with_line_numbers() {
    let cases = [
        ("[colors]\nbrand blue\n", "expected `key = value`"),
        ("[colors]\n\"\" = \"#123456\"\n", "invalid key"),
        ("[colors]\nbr and = \"#123456\"\n", "invalid key"),
        (
            "[options]\nspacing-scale = \"ems\"\n",
            "must be \"px\" or \"tailwind\"",
        ),
        ("[options]\nnope = \"x\"\n", "unknown option `nope`"),
        (
            "[options]\nspacing-scale = { expr = \"x\" }\n",
            "expects a string",
        ),
        ("[radius]\ncard = \"big\"\n", "must be a length"),
        (
            "[opacity]\nmuted = \"120%\"\n",
            "must be an opacity between 0 and 1",
        ),
        (
            "[shadow]\ncard = \"shadow\"\n",
            "expects an inline Rust expression",
        ),
        (
            "[colors]\nbrand = { colour = \"x\" }\n",
            "unknown inline key",
        ),
        (
            "[colors]\nbrand = { expr }\n",
            "inline tables must look like",
        ),
        ("[colors]\nbrand = #123456\n", "expected a quoted string"),
        ("brand = \"#123456\"\n", "must be inside a section"),
        ("[options]\npreset = \"other\"\n", "unknown preset"),
        (
            "[options]\npreset = \"gpui-component\"\ncontext = \"cx.(\"\n",
            "not a Rust expression",
        ),
    ];
    for (source, expected) in cases {
        let error = Theme::parse(source).unwrap_err();
        assert!(error.contains(expected), "{source:?}: {error}");
        assert!(error.starts_with("rsx.toml"), "{source:?}: {error}");
    }
}

#[test]
fn explicit_colors_override_preset_values() {
    let theme =
        Theme::parse("[options]\npreset = \"gpui-component\"\n[colors]\nprimary = \"#123456\"\n")
            .unwrap();
    assert!(matches!(
        theme.colors.get("primary"),
        Some(TokenValue::Color {
            hex: 0x123456,
            alpha: false
        })
    ));
    // Preset colors that were not overridden still resolve through the context.
    assert!(matches!(
        theme.colors.get("muted-foreground"),
        Some(TokenValue::Expr(_))
    ));
}

#[test]
fn class_names_include_radius_tokens() {
    let theme = Theme::parse("[radius]\ncard = \"12px\"\n").unwrap();
    assert!(theme.class_names().contains(&"rounded-card".to_owned()));
}

#[test]
fn load_from_dir_reads_once_and_tracks_the_path() {
    let root = std::env::temp_dir().join(format!("rsx-theme-load-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();

    // No config file: no theme, no error.
    assert!(Theme::load_from_dir(&root).unwrap().is_none());

    std::fs::write(root.join(CONFIG_FILE), "[colors]\nbrand = \"#123456\"\n").unwrap();
    let theme = Theme::load_from_dir(&root).unwrap().unwrap();
    assert_eq!(
        theme.path.as_deref(),
        Some(
            std::fs::canonicalize(root.join(CONFIG_FILE))
                .unwrap()
                .as_path()
        )
    );
    assert!(theme.expand_class("bg-brand").is_some());

    // A second call hits the cache and returns the same shared allocation.
    let again = Theme::load_from_dir(&root).unwrap().unwrap();
    assert!(std::sync::Arc::ptr_eq(&theme, &again));

    // Rebuild tracking points at the config file.
    let tracker = rebuild_tracker(&theme).unwrap().to_string();
    assert!(tracker.contains("include_bytes"), "{tracker}");
    assert!(tracker.contains("rsx.toml"), "{tracker}");
    assert!(rebuild_tracker(&Theme::default()).is_none());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn load_from_dir_reloads_when_content_changes() {
    let root = std::env::temp_dir().join(format!("rsx-theme-fingerprint-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(CONFIG_FILE);
    std::fs::write(&path, "[colors]\nbrand = \"#123456\"\n").unwrap();
    let first = Theme::load_from_dir(&root).unwrap().unwrap();

    // Keep the same path and shape while changing the value. Content fingerprints make
    // this reliable even on filesystems with coarse modification-time resolution.
    std::fs::write(&path, "[colors]\nbrand = \"#654321\"\n").unwrap();
    let second = Theme::load_from_dir(&root).unwrap().unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(
        second.colors().get("brand"),
        Some(&TokenValue::Color {
            hex: 0x654321,
            alpha: false,
        })
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stable_theme_builder_creates_in_memory_tokens() {
    let theme = Theme::builder()
        .scale(SpacingScale::Tailwind)
        .color(
            "accent",
            TokenValue::Color {
                hex: 0x123456,
                alpha: false,
            },
        )
        .spacing("gutter", TokenValue::Length("px(12.0)".into()))
        .build();

    assert_eq!(theme.scale(), Some(SpacingScale::Tailwind));
    assert!(theme.expand_class("bg-accent").is_some());
    assert!(theme.expand_class("gap-gutter").is_some());
}
