use super::*;
use quote::quote;

#[test]
fn rejects_unsupported_function_signatures_and_contexts() {
    for (source, expected) in [
        (
            quote! { async fn Card() -> impl IntoElement { div() } },
            "synchronous",
        ),
        (quote! { fn Card() {} }, "must return an element"),
        (
            quote! { fn Card((x, y): (u8, u8)) -> impl IntoElement { div() } },
            "destructuring",
        ),
        (
            quote! { fn Card(handler: impl Fn()) -> impl IntoElement { div() } },
            "concrete types",
        ),
        (
            quote! { fn Card(label: &str) -> impl IntoElement { div() } },
            "must own their values",
        ),
        (
            quote! { fn Card(#[prop(default)] #[children] children: Vec<AnyElement>) -> impl IntoElement { div() } },
            "only one",
        ),
        (
            quote! { fn Card(#[children(default)] children: Vec<AnyElement>) -> impl IntoElement { div() } },
            "takes no arguments",
        ),
        (
            quote! { fn Card(#[context] cx: &App) -> impl IntoElement { div() } },
            "&mut Window",
        ),
        (
            quote! { fn Card(#[context] cx: &mut Context<Card>) -> impl IntoElement { div() } },
            "&mut App",
        ),
        (
            quote! { fn Card(#[context] a: &mut App, #[context] b: &mut App) -> impl IntoElement { div() } },
            "only once",
        ),
        (
            quote! { fn Card(#[context] #[prop(default)] cx: &mut App) -> impl IntoElement { div() } },
            "combined",
        ),
    ] {
        let error = expand(TokenStream::new(), source).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn accepts_generics_and_rejects_tuple_structs() {
    let out = expand_component(TokenStream::new(), quote! { struct Card<T> { t: T } }).to_string();
    assert!(!out.contains("compile_error"), "{out}");
    let out = expand_component(TokenStream::new(), quote! { struct Card(u8); }).to_string();
    assert!(out.contains("named fields"));
}

#[test]
fn generates_builder_function_and_markers() {
    let out = expand_component(
        TokenStream::new(),
        quote! {
            pub struct TaskCard {
                title: SharedString,
                #[prop(default)] elevated: bool,
                #[children] children: Vec<AnyElement>,
            }
        },
    )
    .to_string();
    assert!(out.contains("pub fn TaskCard ()"));
    assert!(out.contains("pub struct TaskCardBuilder"));
    assert!(out.contains("mod __rsx_task_card_props"));
    assert!(out.contains("missing required prop `title` on `<TaskCard>`"));
    assert!(out.contains("pub fn elevated_if"));
    assert!(!out.contains("# [prop"));
    assert!(!out.contains("# [children]"));
}

#[test]
fn raw_component_names_generate_valid_items() {
    for source in [
        quote! { struct r#Card { title: u8 } },
        quote! { fn r#Card(title: u8) -> impl IntoElement { div() } },
    ] {
        let out = expand(TokenStream::new(), source).unwrap().to_string();
        assert!(out.contains("CardBuilder"), "{out}");
        assert!(out.contains("mod __rsx_card_props"), "{out}");
        assert!(!out.contains("<r#Card>"), "{out}");
    }
}

#[test]
fn raw_and_colliding_prop_names_get_distinct_valid_generics() {
    let out = expand_component(
        TokenStream::new(),
        quote! { struct Input { r#type: u8, a_b: u8, a__b: u8 } },
    )
    .to_string();
    assert!(out.contains("__Type"));
    assert!(out.contains("__AB ,") || out.contains("__AB>"));
    assert!(out.contains("__AB1"));
}

#[test]
fn rejects_attributes_on_non_components_and_unsupported_items() {
    let out = expand_component(quote! { strict }, quote! { struct Card { title: u8 } }).to_string();
    assert!(out.contains("compile_error"), "{out}");
    assert!(out.contains("takes no arguments"), "{out}");

    for item in [quote! { enum E { A } }, quote! { impl E {} }] {
        let out = expand_component(TokenStream::new(), item).to_string();
        assert!(out.contains("compile_error"), "{out}");
        assert!(
            out.contains("requires a struct with named fields or a synchronous function"),
            "{out}"
        );
    }
}

#[test]
fn cfg_gates_apply_to_every_generated_item() {
    let out = expand(
        TokenStream::new(),
        quote! {
            #[cfg(feature = "cards")]
            struct Card { title: u8 }
        },
    )
    .unwrap()
    .to_string();
    assert!(
        out.matches("# [cfg (feature = \"cards\")]").count() >= 2,
        "{out}"
    );
}

#[test]
fn function_contexts_are_injected_during_render() {
    let out = expand(
        TokenStream::new(),
        quote! {
            fn Card(
                label: String,
                #[context] window: &mut Window,
                #[context] cx: &mut App,
            ) -> impl IntoElement { div() }
        },
    )
    .unwrap()
    .to_string();
    assert!(out.contains("__rsx_render_Card"), "{out}");
    assert!(out.contains("__rsx_window"), "{out}");
    assert!(out.contains("__rsx_cx"), "{out}");
    assert!(!out.contains("# [context]"), "{out}");
    assert!(!out.contains("# [cfg (test)]"), "{out}");
    // Context parameters are not props: only `label` becomes a field setter.
    assert_eq!(out.matches("fn label").count(), 1, "{out}");
}

#[test]
fn function_components_reject_unsupported_signatures() {
    for (source, expected) in [
        (
            quote! { fn Card(self) -> impl IntoElement { div() } },
            "cannot take `self`",
        ),
        (
            quote! { unsafe fn Card() -> impl IntoElement { div() } },
            "safe, synchronous",
        ),
        (
            quote! { const fn Card() -> impl IntoElement { div() } },
            "safe, synchronous",
        ),
        (
            quote! { extern "C" fn Card() -> impl IntoElement { div() } },
            "safe, synchronous",
        ),
        (
            quote! { fn Card(#[context] cx: App) -> impl IntoElement { div() } },
            "#[context] requires",
        ),
        (
            quote! { fn Card(#[unknown] value: u8) -> impl IntoElement { div() } },
            "only #[prop(...)] and #[children]",
        ),
    ] {
        let error = expand(TokenStream::new(), source).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn function_components_keep_generics_used_only_in_the_body() {
    let out = expand(
        TokenStream::new(),
        quote! { fn Card<T>() -> impl IntoElement { div() } },
    )
    .unwrap()
    .to_string();
    assert!(out.contains("__rsx_generics"), "{out}");
    assert!(out.contains("PhantomData"), "{out}");
}

#[test]
fn phantom_generics_avoid_prop_name_collisions() {
    let out = expand(
        TokenStream::new(),
        quote! { fn Card<T>(__rsx_generics: u8) -> impl IntoElement { div() } },
    )
    .unwrap()
    .to_string();
    assert!(out.contains("__rsx_generics_"), "{out}");
}

#[test]
fn function_generics_gain_static_bounds_for_render_once() {
    let out = expand(
        TokenStream::new(),
        quote! { fn Card<T>() -> impl IntoElement where T: Clone { div() } },
    )
    .unwrap()
    .to_string();
    assert!(out.contains("T : Clone + 'static"), "{out}");

    let bounded = expand(
        TokenStream::new(),
        quote! { fn Card<T: Clone>() -> impl IntoElement { div() } },
    )
    .unwrap()
    .to_string();
    assert!(bounded.contains("T : Clone + 'static"), "{bounded}");

    let already_static = expand(
        TokenStream::new(),
        quote! { fn Card<T: 'static>() -> impl IntoElement { div() } },
    )
    .unwrap()
    .to_string();
    assert!(
        !already_static.contains("'static + 'static"),
        "{already_static}"
    );

    let lifetime = expand(
        TokenStream::new(),
        quote! { fn Card<'a>() -> impl IntoElement { div() } },
    )
    .unwrap()
    .to_string();
    assert!(lifetime.contains("'a : 'static"), "{lifetime}");
}

#[test]
fn function_docs_and_deprecation_move_to_the_constructor() {
    let out = expand(
        TokenStream::new(),
        quote! {
            /// Renders a card.
            #[deprecated(note = "use Panel")]
            fn Card(title: String) -> impl IntoElement { div() }
        },
    )
    .unwrap()
    .to_string();
    assert!(out.contains("Renders a card."), "{out}");
    assert!(
        out.contains("# [deprecated (note = \"use Panel\")]"),
        "{out}"
    );
    assert!(out.contains("struct Card"), "{out}");
    // The private render function is not documented or deprecated; the constructor is.
    assert!(!out.contains("__rsx_render_Card # [doc"), "{out}");
    assert!(
        !out.contains("Renders a card.\"] fn __rsx_render_Card"),
        "{out}"
    );
}
