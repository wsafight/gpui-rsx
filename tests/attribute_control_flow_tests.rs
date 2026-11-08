mod common;

use common::*;
use gpui_rsx::rsx;

fn visible_early_return(early: bool, continued: &mut bool) -> MockElement {
    let element = rsx! {
        <div visible={{
            if early { return div(); }
            true
        }} />
    };
    *continued = true;
    element
}

fn static_if_early_return(early: bool, continued: &mut bool) -> MockElement {
    let element = rsx! {
        <div class={if {
            if early { return div(); }
            true
        } { "flex" } else { "grid" }} />
    };
    *continued = true;
    element
}

fn static_match_early_return(early: bool, continued: &mut bool) -> MockElement {
    let element = rsx! {
        <div class={match {
            if early { return div(); }
            0
        } {
            0 => "flex",
            _ => "grid",
        }} />
    };
    *continued = true;
    element
}

fn dynamic_class_early_return(early: bool, continued: &mut bool) -> MockElement {
    let element = rsx! {
        <div class={{
            if early { return div(); }
            "flex"
        }} />
    };
    *continued = true;
    element
}

fn assert_outer_early_return(build: fn(bool, &mut bool) -> MockElement) {
    let mut continued = false;
    let _ = build(true, &mut continued);
    assert!(!continued, "return must exit the outer function");

    let mut continued = false;
    let _ = build(false, &mut continued);
    assert!(continued, "the normal path must continue");
}

#[test]
fn attribute_expressions_return_from_the_outer_function() {
    assert_outer_early_return(visible_early_return);
    assert_outer_early_return(static_if_early_return);
    assert_outer_early_return(static_match_early_return);
    assert_outer_early_return(dynamic_class_early_return);
}

fn handler() {}

#[test]
fn attribute_evaluation_order_and_single_evaluation_are_preserved() {
    let mut order = Vec::new();
    let _ = rsx! {
        <div
            base={{ order.push("construct"); div() }}
            class={{ order.push("class"); "gap-7" }}
            onUnknownClass={{
                order.push("unknown-handler");
                |_: &str| {}
            }}
            visible={{ order.push("visible"); true }}
            onMouseMove={{ order.push("handler"); handler }}
        >
            {{ order.push("child"); "child" }}
        </div>
    };

    assert_eq!(
        order,
        [
            "construct",
            "unknown-handler",
            "class",
            "visible",
            "handler",
            "child",
        ]
    );
}

#[test]
fn class_conditions_are_evaluated_once_and_unselected_branches_stay_lazy() {
    let mut if_conditions = 0;
    let mut else_if_conditions = 0;
    let _ = rsx! {
        <div class={if { if_conditions += 1; true } {
            "flex"
        } else if { else_if_conditions += 1; true } {
            "grid"
        } else {
            "block"
        }} />
    };
    assert_eq!(if_conditions, 1);
    assert_eq!(else_if_conditions, 0);

    let mut match_expressions = 0;
    let mut later_guards = 0;
    let _ = rsx! {
        <div class={match { match_expressions += 1; 0 } {
            0 => "flex",
            n if { later_guards += 1; n == 1 } => "grid",
            _ => "block",
        }} />
    };
    assert_eq!(match_expressions, 1);
    assert_eq!(later_guards, 0);

    let mut visible_evaluations = 0;
    let _ = rsx! {
        <div visible={{ visible_evaluations += 1; true }} />
    };
    assert_eq!(visible_evaluations, 1);

    let choose_then = false;
    let mut dynamic_evaluations = 0;
    let mut then_evaluations = 0;
    let mut else_evaluations = 0;
    let _ = rsx! {
        <div class={{
            dynamic_evaluations += 1;
            if choose_then {
                { then_evaluations += 1; "gap-7" }
            } else {
                { else_evaluations += 1; "gap-9" }
            }
        }} />
    };
    assert_eq!(dynamic_evaluations, 1);
    assert_eq!(then_evaluations, 0);
    assert_eq!(else_evaluations, 1);
}
