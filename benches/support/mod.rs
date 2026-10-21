#![allow(dead_code)]

//! Mock GPUI builders used by the class benchmarks.

use std::hint::black_box;

// Mock GPUI 类型（与 tests/macro_tests.rs 相同）
#[derive(Debug, Default)]
pub(crate) struct MockElement {
    pub(crate) checksum: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FontWeight;

impl FontWeight {
    pub(crate) const THIN: FontWeight = FontWeight;
    pub(crate) const EXTRA_LIGHT: FontWeight = FontWeight;
    pub(crate) const LIGHT: FontWeight = FontWeight;
    pub(crate) const NORMAL: FontWeight = FontWeight;
    pub(crate) const MEDIUM: FontWeight = FontWeight;
    pub(crate) const SEMIBOLD: FontWeight = FontWeight;
    pub(crate) const BOLD: FontWeight = FontWeight;
    pub(crate) const EXTRA_BOLD: FontWeight = FontWeight;
    pub(crate) const BLACK: FontWeight = FontWeight;
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

pub(crate) type JustifyContent = AlignContent;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum AlignContent {
    Stretch,
    SpaceEvenly,
}

#[allow(dead_code)]
#[derive(Default, Debug)]
pub(crate) struct StyleRefinement {
    pub(crate) align_items: Option<AlignItems>,
    pub(crate) align_self: Option<AlignItems>,
    pub(crate) justify_content: Option<JustifyContent>,
    pub(crate) align_content: Option<AlignContent>,
    pub(crate) aspect_ratio: Option<f32>,
    pub(crate) flex_grow: Option<f32>,
}

pub(crate) fn div() -> MockElement {
    MockElement {
        checksum: black_box(0),
    }
}

pub(crate) fn rgb(hex: u32) -> u32 {
    hex
}
pub(crate) fn rgba(hex: u32) -> u32 {
    hex
}

pub(crate) fn px(val: f32) -> f32 {
    val
}
pub(crate) fn rems(val: f32) -> f32 {
    val
}
pub(crate) fn relative(val: f32) -> f32 {
    val
}

macro_rules! mock_no_arg_methods {
    ($($name:ident),* $(,)?) => {
        $(fn $name(self) -> Self { self })*
    };
}

macro_rules! mock_value_methods {
    ($($name:ident),* $(,)?) => {
        $(fn $name<T>(self, _: T) -> Self { self })*
    };
}

mod element;
mod styled;

pub(crate) use styled::Styled;
