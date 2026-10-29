//! Local component signatures for editor hints; this is deliberately not Rust type resolution.
use crate::{Diagnostic, DiagnosticSeverity, byte_offset, walk_body};
use gpui_rsx_core::syntax::{RsxAttribute, RsxBody};
use quote::ToTokens;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentProp {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub flag: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub props: Vec<ComponentProp>,
}

fn annotated(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().segments.last().is_some_and(|s| s.ident == name))
}

/// `#[prop(default)]` / `#[prop(default = ..)]`; `#[prop(callback)]` alone stays required.
fn has_default(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("prop"))
        .any(|attr| {
            let mut default = false;
            let _ = attr.parse_nested_meta(|meta| {
                default |= meta.path.is_ident("default");
                if meta.input.peek(syn::Token![=]) {
                    meta.value()?.parse::<syn::Expr>()?;
                }
                Ok(())
            });
            default
        })
}

fn prop(name: &syn::Ident, ty: &syn::Type, attrs: &[syn::Attribute]) -> Option<ComponentProp> {
    if annotated(attrs, "children") {
        return None;
    }
    let default = has_default(attrs);
    Some(ComponentProp {
        name: name.to_string(),
        ty: ty.to_token_stream().to_string(),
        required: !default,
        flag: default && matches!(ty, syn::Type::Path(path) if path.path.is_ident("bool")),
    })
}

pub(crate) fn collect(source: &str) -> Vec<Component> {
    fn items(entries: &[syn::Item], prefix: &str, out: &mut Vec<Component>) {
        for item in entries {
            let (name, props) = match item {
                syn::Item::Struct(item) if annotated(&item.attrs, "component") => (
                    &item.ident,
                    item.fields
                        .iter()
                        .filter_map(|f| prop(f.ident.as_ref()?, &f.ty, &f.attrs))
                        .collect(),
                ),
                syn::Item::Fn(item) if annotated(&item.attrs, "component") => (
                    &item.sig.ident,
                    item.sig
                        .inputs
                        .iter()
                        .filter_map(|arg| {
                            let syn::FnArg::Typed(arg) = arg else {
                                return None;
                            };
                            let syn::Pat::Ident(name) = &*arg.pat else {
                                return None;
                            };
                            // Window/App parameters are injected by the component macro.
                            if annotated(&arg.attrs, "context") {
                                return None;
                            }
                            prop(&name.ident, &arg.ty, &arg.attrs)
                        })
                        .collect(),
                ),
                syn::Item::Mod(item) => {
                    if let Some((_, content)) = &item.content {
                        items(content, &format!("{prefix}{}::", item.ident), out);
                    }
                    continue;
                }
                _ => continue,
            };
            out.push(Component {
                name: format!("{prefix}{name}"),
                props,
            });
        }
    }
    let mut result = Vec::new();
    if let Ok(file) = syn::parse_file(source) {
        items(&file.items, "", &mut result);
    }
    result
}

pub fn find<'a>(components: &'a [Component], name: &str) -> Option<&'a Component> {
    if let Some(exact) = components.iter().find(|c| c.name == name) {
        return Some(exact);
    }
    let mut matches = components
        .iter()
        .filter(|c| c.name.rsplit("::").next() == Some(name));
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

pub(crate) fn diagnostics(
    source: &str,
    body: &RsxBody,
    components: &[Component],
    out: &mut Vec<Diagnostic>,
) {
    walk_body(body, &mut |el| {
        let name = el.name.to_string();
        let Some(component) = find(components, &name) else {
            return;
        };
        // A custom builder may already contain props; avoid false missing-prop reports.
        if el
            .attributes
            .iter()
            .any(|a| matches!(a, RsxAttribute::Value {name, ..} if name == "base"))
        {
            return;
        }
        let missing: Vec<_> = component
            .props
            .iter()
            .filter(|p| {
                p.required
                    && !el.attributes.iter().any(|a| match a {
                        RsxAttribute::Value { name, .. } | RsxAttribute::Flag(name) => {
                            prop_name(name) == p.name
                        }
                        _ => false,
                    })
            })
            .collect();
        if missing.is_empty() {
            return;
        }
        let additions = missing
            .iter()
            .map(|p| format!(" {}={{todo!(\"provide {}\")}}", p.name, p.name))
            .collect::<String>();
        out.push(Diagnostic {
            start: byte_offset(source, el.name.span(), false),
            end: byte_offset(source, el.name.span(), true),
            severity: DiagnosticSeverity::Error,
            message: format!(
                "Missing required prop(s) {} on `<{name}>`",
                missing
                    .iter()
                    .map(|p| format!("`{}`", p.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            suggestion: Some(format!("{name}{additions}")),
        });
    });
}

fn prop_name(name: &syn::Ident) -> String {
    let mut snake = String::new();
    for (i, c) in name.to_string().chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
    }
    snake
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiagnosticSeverity;

    const SOURCE: &str = r#"
        #[component]
        #[derive(IntoElement)]
        pub struct Card {
            /// Shown at the top.
            title: String,
            #[prop(default)]
            elevated: bool,
            #[prop(default = 2)]
            depth: u8,
            #[children]
            children: Vec<AnyElement>,
        }

        #[component]
        fn Badge(
            label: String,
            #[prop(default)] active: bool,
            #[context] cx: &mut App,
        ) -> impl IntoElement { div() }

        mod ui {
            #[component]
            pub struct Row { id: u8 }
        }
    "#;

    fn body(source: &str) -> RsxBody {
        syn::parse_str(source).unwrap()
    }

    #[test]
    fn callback_props_stay_required_unless_they_have_a_default() {
        let components = collect(
            "#[component] fn Button(#[prop(callback)] on_press: Rc<dyn OnPress>, \
             #[prop(callback, default = noop())] on_hover: Rc<dyn OnPress>) -> impl IntoElement { div() }",
        );
        let required: Vec<_> = components[0].props.iter().map(|p| p.required).collect();
        assert_eq!(required, [true, false]);
    }

    #[test]
    fn collects_struct_function_and_nested_components() {
        let components = collect(SOURCE);
        let names: Vec<_> = components.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Card", "Badge", "ui::Row"]);

        assert_eq!(
            components[0].props,
            vec![
                ComponentProp {
                    name: "title".into(),
                    ty: "String".into(),
                    required: true,
                    flag: false,
                },
                ComponentProp {
                    name: "elevated".into(),
                    ty: "bool".into(),
                    required: false,
                    flag: true,
                },
                ComponentProp {
                    name: "depth".into(),
                    ty: "u8".into(),
                    required: false,
                    flag: false,
                },
            ]
        );
        // `#[context]` parameters are injected by the macro, not props.
        assert_eq!(components[1].props.len(), 2);
        assert_eq!(components[2].name, "ui::Row");
    }

    #[test]
    fn ignores_items_that_are_not_components() {
        let source = "struct Plain { x: u8 } fn helper() {} #[component] enum E { A }";
        assert!(collect(source).is_empty());
        assert!(collect("not rust").is_empty());
    }

    #[test]
    fn find_prefers_exact_then_unique_short_names() {
        let components = collect(SOURCE);
        assert_eq!(find(&components, "Card").unwrap().name, "Card");
        assert_eq!(find(&components, "Row").unwrap().name, "ui::Row");
        assert!(find(&components, "Missing").is_none());

        let ambiguous = vec![
            Component {
                name: "a::Row".into(),
                props: vec![],
            },
            Component {
                name: "b::Row".into(),
                props: vec![],
            },
        ];
        assert!(find(&ambiguous, "Row").is_none());
        assert_eq!(find(&ambiguous, "a::Row").unwrap().name, "a::Row");
    }

    #[test]
    fn reports_missing_required_props_with_a_suggestion() {
        let components = collect(SOURCE);
        let mut out = Vec::new();
        diagnostics("<Card/>", &body("<Card/>"), &components, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].severity, DiagnosticSeverity::Error);
        assert!(out[0].message.contains("`title`"), "{}", out[0].message);
        assert!(
            out[0].suggestion.as_deref().unwrap().contains("title={"),
            "{:?}",
            out[0].suggestion
        );
        assert_eq!(&"<Card/>"[out[0].start..out[0].end], "Card");
    }

    #[test]
    fn satisfied_props_and_base_builders_are_not_reported() {
        let components = collect(SOURCE);
        let mut out = Vec::new();
        diagnostics(
            "<Card title={name}/>",
            &body("<Card title={name}/>"),
            &components,
            &mut out,
        );
        assert!(out.is_empty());

        // camelCase attributes map to snake_case props.
        diagnostics(
            "<Card title={name}/>",
            &body("<Card title={name}/>"),
            &components,
            &mut out,
        );
        assert!(out.is_empty());

        // `base` already seeded the builder, so missing props may be intentional.
        let mut out = Vec::new();
        diagnostics(
            "<Card base={Card::new()}/>",
            &body("<Card base={Card::new()}/>"),
            &components,
            &mut out,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn camel_case_attribute_names_match_snake_case_props() {
        let components = collect("#[component] struct Form { first_name: String }");
        let mut out = Vec::new();
        diagnostics(
            "<Form firstName={name}/>",
            &body("<Form firstName={name}/>"),
            &components,
            &mut out,
        );
        assert!(out.is_empty(), "{out:?}");

        let mut out = Vec::new();
        diagnostics("<Form/>", &body("<Form/>"), &components, &mut out);
        assert_eq!(out.len(), 1);
        assert!(
            out[0].message.contains("`first_name`"),
            "{}",
            out[0].message
        );
    }

    #[test]
    fn flag_props_count_as_present() {
        let components = collect("#[component] struct Card { title: String }");
        let mut out = Vec::new();
        diagnostics(
            "<Card title />",
            &body("<Card title />"),
            &components,
            &mut out,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn walks_nested_fragments_and_loops() {
        let components = collect("#[component] struct Card { title: String }");
        let source = "<><div>{for row in rows { <Card/> }}</div></>";
        let mut out = Vec::new();
        diagnostics(source, &body(source), &components, &mut out);
        assert_eq!(out.len(), 1);
    }
}
