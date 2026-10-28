use gpui::prelude::*;
use gpui::*;
use gpui_rsx::{component, rsx};
use std::sync::{Arc, Mutex};

type Observations = Arc<Mutex<Vec<(String, bool, usize)>>>;

#[component]
fn Badge(
    label: SharedString,
    observations: Observations,
    #[prop(default)] elevated: bool,
    #[prop(default = 2usize)] depth: usize,
    #[children] children: Vec<AnyElement>,
) -> impl IntoElement {
    observations
        .lock()
        .unwrap()
        .push((format!("{label}:{depth}"), elevated, children.len()));
    rsx! { <div>{label}{...children}</div> }
}

mod ui {
    use super::*;

    #[component]
    pub fn Contextual(
        observed: Arc<Mutex<bool>>,
        #[context] window: &mut Window,
        #[context] cx: &mut App,
    ) -> impl IntoElement {
        let _ = (window, cx);
        *observed.lock().unwrap() = true;
        div()
    }

    #[component]
    pub fn Empty() -> Div {
        div()
    }

    #[component]
    pub fn Callback(callback: Box<dyn Fn()>, text: &'static str) -> impl IntoElement {
        callback();
        div().child(text)
    }
}

#[test]
fn functions_render_lazily_with_defaults_and_children() {
    let observations = Observations::default();
    let badge = Badge()
        .observations(observations.clone())
        .label("status")
        .elevated()
        .child(div())
        .children([div(), div()])
        .build();
    assert!(observations.lock().unwrap().is_empty());
    let _ = badge.render(&mut Window, &mut App);
    assert_eq!(
        *observations.lock().unwrap(),
        vec![("status:2".into(), true, 3)]
    );
}

#[test]
fn functions_share_optional_prop_setters_with_struct_components() {
    let observations = Observations::default();
    let _ = Badge()
        .label(String::from("draft"))
        .observations(observations.clone())
        .depth(4usize)
        .elevated_if(false)
        .build()
        .render(&mut Window, &mut App);
    assert_eq!(
        *observations.lock().unwrap(),
        vec![("draft:4".into(), false, 0)]
    );
}

#[test]
fn functions_receive_context_only_when_declared() {
    let observed = Arc::new(Mutex::new(false));
    let _ = ui::Contextual()
        .observed(observed.clone())
        .build()
        .render(&mut Window, &mut App);
    assert!(*observed.lock().unwrap());
    let _ = rsx! {
        <div>
            <ui::Empty />
            <ui::Contextual observed={observed} />
            <Badge label="nested" observations={Observations::default()}>
                <><span>{"one"}</span><ui::Empty /></>
            </Badge>
        </div>
    };
}

#[test]
fn owned_callbacks_and_static_references_are_props() {
    let observed = Arc::new(Mutex::new(false));
    let captured = observed.clone();
    let callback: Box<dyn Fn()> = Box::new(move || *captured.lock().unwrap() = true);
    let _ = ui::Callback()
        .callback(callback)
        .text("static")
        .build()
        .render(&mut Window, &mut App);
    assert!(*observed.lock().unwrap());
}

#[test]
fn nested_fragments_compile_in_loops_spreads_and_root_fragments() {
    let rows = [vec![1u8, 2], vec![3u8]];
    let _ = rsx! {
        <div>
            <></>
            {for row in &rows {
                <>
                    <ui::Empty />
                    <>{for item in row { <span>{item.to_string()}</span> }}</>
                    {...[div(), div()]}
                </>
            }}
            {for _ in 0..2 { <></> }}
        </div>
    };
    let siblings = rsx! { <><div/><><div/><><div/></></></> };
    assert_eq!(siblings.len(), 3);
}
