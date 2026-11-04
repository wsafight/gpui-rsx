#[cfg(test)]
mod tests {
    use engine::{
        AnyElement, Element, ElementId, FontWeight, SharedString, StyleRefinement, div, prelude::*,
        px, relative, rems, rgb, rgba,
    };
    use gpui_rsx::{component, rsx, rsx_fragment, rsx_listener, rsx_update};

    fn renamed_dependency_supports_listeners(cx: &mut engine::Context<()>) {
        let _ = rsx! { <div onClick={rsx_listener!(cx, notify, |_, _, _, _| {})} /> };
        let _ = rsx! { <div onClick={rsx_listener!(cx, |_, _, _, _| {})} /> };
    }

    fn renamed_dependency_supports_updates(view: &mut (), cx: &mut engine::Context<()>) {
        rsx_update!(view, cx, |_, _| {});
    }

    mod shared {
        gpui_rsx::rsx_runtime!(::engine);
    }

    mod default_shared {
        gpui_rsx::rsx_runtime!();
    }

    #[component]
    fn Card(
        title: SharedString,
        #[prop(default)] compact: bool,
        #[children] children: Vec<AnyElement>,
    ) -> impl IntoElement {
        rsx! { <div whenClass={(compact, "p-2")}>{title}{...children}</div> }
    }

    #[component]
    #[deprecated(note = "use Card")]
    fn TypedCard<Value: Into<SharedString> + 'static>(title: Value) -> impl IntoElement {
        div().child(title.into())
    }

    #[test]
    #[allow(deprecated)]
    fn renamed_dependency_supports_generic_and_deprecated_components() {
        let props = TypedCard::<SharedString>()
            .title(SharedString::from("Inbox"))
            .build();
        assert_eq!(props.title, "Inbox");
        let _ = props.into_any_element();
    }

    #[test]
    fn renamed_dependency_supports_components_ids_and_erased_fragments() {
        let _ = renamed_dependency_supports_listeners;
        let _ = renamed_dependency_supports_updates;
        let props = rsx! { <Card title="Inbox"><div /></Card> }.build();
        assert_eq!(props.title, "Inbox");
        assert!(!props.compact);
        assert_eq!(props.children.len(), 1);

        let interactive = rsx! { <div onClick={|_, _, _| {}} /> };
        assert!(Element::id(&interactive).is_some());

        let key = String::from("row");
        let keyed = rsx! { <div key={key} onClick={|_, _, _| {}} /> };
        assert!(matches!(Element::id(&keyed), Some(ElementId::Name(_))));

        let parts = rsx_fragment! { <><div /><div id="stateful" /></> };
        assert_eq!(parts.len(), 2);
        let empty = rsx! { <>{for _ in 0..0 {}}</> };
        assert!(empty.is_empty());
    }

    #[test]
    fn renamed_dependency_supports_inline_and_explicit_shared_runtimes() {
        let classes = String::from("p-4 w-[8px]");
        let mut actual = rsx! { <div class={classes} /> };
        let mut expected = div().p(px(4.)).w(px(8.));
        assert_eq!(
            format!("{:?}", actual.style()),
            format!("{:?}", expected.style())
        );

        for (helper, padding) in [
            (shared::rsx_runtime::px as ApplyClass, 4.),
            (shared::rsx_runtime::tailwind as ApplyClass, 16.),
            (default_shared::rsx_runtime::px as ApplyClass, 4.),
            (default_shared::rsx_runtime::tailwind as ApplyClass, 16.),
        ] {
            let actual = helper(StyleRefinement::default(), "p-4", &mut |class| {
                panic!("unexpected class: {class}")
            });
            let mut expected = div().p(px(padding));
            assert_eq!(format!("{actual:?}"), format!("{:?}", expected.style()));
        }
    }

    type ApplyClass = fn(StyleRefinement, &str, &mut dyn FnMut(&str)) -> StyleRefinement;
}
