//! Code generator
//!
//! Converts parsed RSX into GPUI code
//!
//! Generates idiomatic GPUI method chaining patterns:
//! ```ignore
//! div().id("auto_0").flex().bg(rgb(0xff)).on_click(handler).child("text")
//! ```
//!
//! # Module Structure
//!
//! - `tables`: Static lookup tables (colors, events, attribute mappings, etc.)
//! - `class`: CSS class string parsing, including unified color handling
//! - `attribute`: RSX attribute to GPUI method conversion
//! - `element`: Code generation for elements and child nodes

pub(crate) mod attribute;
pub(crate) mod class;
pub(crate) mod element;
pub(crate) mod runtime;
pub(crate) mod tables;
pub mod data_table_parser;
