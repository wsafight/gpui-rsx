//! Invalid RSX forms must fail against the same GPUI dependency as the runnable gallery.
//!
//! Tags must match, and text must be a string or a Rust expression.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div></span> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div>unquoted text</div> };
//! ```
//!
//! Special constructors require their arguments.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <img /> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <canvas prepaint={|_, _, _| ()} /> };
//! ```
//!
//! Every stateful descendant of a loop requires an explicit identity.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div>{for n in 0..2 { <button onClick={|_, _, _| {}} /> }}</div> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div>{for n in 0..2 { <><section><button class="active:opacity-75" /></section></> }}</div> };
//! ```
//!
//! Conditional classes require static, non-stateful strings.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let classes = "p-4";
//! let _ = rsx! { <div whenClass={(true, classes)} /> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div whenClass={(true, "overflow-scroll")} /> };
//! ```
//!
//! Browser-only and unsupported generic attributes are diagnosed explicitly.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div whiteSpace="nowrap" /> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div groupDragOver={|style| style} /> };
//! ```
//!
//! Strict unknown classes and malformed lengths cannot silently change layout.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx_strict;
//! let _ = rsx_strict! { <div class="not-a-supported-class" /> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui_rsx::rsx;
//! let _ = rsx! { <div class="w-[oops]" /> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui::{div, prelude::*};
//! use gpui_rsx::rsx_strict;
//! let _ = rsx_strict! { <div class="grid-cols-70000" /> };
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui::{div, prelude::*};
//! use gpui_rsx::rsx_strict;
//! let _ = rsx_strict! { <div class="row-start-40000" /> };
//! ```
//!
//! Component required props remain checked by Rust's type system.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui::{div, prelude::*};
//! use gpui_rsx::{component, rsx};
//! #[component]
//! fn Label(title: gpui::SharedString) -> impl IntoElement { div().child(title) }
//! let _ = rsx! { <Label /> }.into_any_element();
//! ```
//!
//! Parameters must be named props; context parameters must be mutable GPUI contexts.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui::{div, prelude::*};
//! use gpui_rsx::component;
//! #[component]
//! fn Pair((left, right): (u8, u8)) -> impl IntoElement { div() }
//! ```
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui::{div, prelude::*};
//! use gpui_rsx::component;
//! #[component]
//! fn Context(#[context] cx: &gpui::App) -> impl IntoElement { div() }
//! ```
//!
//! Callback annotations need a written trait object so closure arguments can infer.
//! ```compile_fail
//! # use gpui::prelude::*;
//! # use gpui::*;
//! use gpui::{div, prelude::*};
//! use gpui_rsx::component;
//! type Callback = std::rc::Rc<dyn Fn()>;
//! #[component]
//! fn Button(#[prop(callback)] callback: Callback) -> impl IntoElement { div() }
//! ```
