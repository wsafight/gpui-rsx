//! Snapshot tests of macro expansions (`cargo insta review` to accept changes).
//!
//! Every case is expanded with the real code generator and pretty-printed, so a refactor
//! that changes generated code shows up as a readable diff.

use gpui_rsx_core::{ClassMode, SpacingScale, Theme, expand_str};

fn pretty(source: &str, mode: ClassMode) -> String {
    let tokens = expand_str(source, mode).unwrap_or_else(|e| panic!("{source}: {e}"));
    let file: syn::File = syn::parse_quote! {
        fn expanded() {
            let _ = #tokens;
        }
    };
    prettyplease::unparse(&file)
}

macro_rules! snapshot {
    ($name:ident, $mode:expr, $source:expr) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!(stringify!($name), pretty($source, $mode), $source);
        }
    };
    ($name:ident, $source:expr) => {
        snapshot!($name, ClassMode::PERMISSIVE, $source);
    };
}

snapshot!(
    static_classes,
    r#"<div class="flex flex-col gap-4 p-4 bg-blue-500 text-white rounded-md" />"#
);
snapshot!(
    arbitrary_values,
    r#"<div class="w-[280px] h-[18rem] w-6/24 bg-[#f00] text-[rgba(15,23,42,0.8)]" />"#
);
snapshot!(
    state_variants,
    r#"<button class="bg-blue-500 hover:bg-blue-600 hover:text-white active:opacity-75 group-hover/row:underline" />"#
);
snapshot!(
    state_class_attributes,
    r#"<div hoverClass="bg-zinc-800" focusClass="border-blue-500" />"#
);
snapshot!(
    event_auto_id,
    r#"<button onClick={handler}>{"Save"}</button>"#
);
snapshot!(
    keyed_loop,
    r#"<ul>{for item in items { <li key={item.id} onClick={handler}>{item.name.clone()}</li> }}</ul>"#
);
snapshot!(
    fragment_children_spread,
    r#"<><div>{"a"}{"b"}</div><div>{...rows}</div></>"#
);
snapshot!(
    conditionals,
    r#"<div when={(active, |el| el.flex())} whenSome={(width, |el, w| el.w(px(w)))} whenClass={(selected, "bg-blue-50")} />"#
);
snapshot!(
    static_branch_class,
    r#"<div class={if active { "flex gap-2" } else { "hidden" }} />"#
);
snapshot!(
    styled_heading,
    r#"<h1 styled class="text-zinc-100">{"Title"}</h1>"#
);
snapshot!(
    base_component,
    r#"<Button base={Button::new("save")} label={"Save"} small />"#
);
snapshot!(
    special_tags,
    r#"<div><img src={"logo.png"} /><svg src={"icon.svg"} class="size-4" /></div>"#
);
snapshot!(
    unknown_classes_permissive,
    r#"<div class="flex flex-colum md:flex-row dark:bg-black h-flex" />"#
);
snapshot!(
    unknown_classes_strict,
    ClassMode::STRICT,
    r#"<div class="flex flex-colum" />"#
);
snapshot!(
    tailwind_scale,
    ClassMode::PERMISSIVE.with_scale(SpacingScale::Tailwind),
    r#"<div class="gap-4 p-2.5 w-64 gap-[14px]" />"#
);

#[test]
fn theme_tokens() {
    let theme = std::sync::Arc::new(
        Theme::parse(
            "[colors]\nbrand = \"#2563eb\"\nsurface = { expr = \"cx.theme().background\" }\n[radius]\ncard = \"12px\"\n[spacing]\ngutter = \"18px\"\n",
        )
        .unwrap(),
    );
    let source = r#"<div class="bg-brand text-surface rounded-card p-gutter hover:bg-surface" />"#;
    insta::assert_snapshot!(
        "theme_tokens",
        pretty(source, ClassMode::PERMISSIVE.with_theme(Some(theme))),
        source
    );
}

#[test]
fn dynamic_class_helper_shape() {
    // The helper is large; snapshot only its call-site wrapper to keep the diff focused.
    let expanded = pretty(r#"<div class={classes} />"#, ClassMode::STRICT);
    let tail: String = expanded
        .lines()
        .skip_while(|l| !l.contains("fn __rsx_unknown_class_handler"))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("dynamic_class_call_site", tail);
}
