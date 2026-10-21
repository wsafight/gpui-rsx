//! rsx! 宏展开测试
//!
//! 通过 mock GPUI 类型，实际展开 rsx! 宏验证生成代码的正确性。
//! 每个测试都会真实调用 rsx! 宏，确认展开后的方法链可以编译。

mod common;

use common::*;
use gpui_rsx::rsx;

macro_rules! class_compile_tests {
    ($($name:ident => $classes:literal),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                let _el = rsx! { <div class=$classes /> };
            }
        )+
    };
}

macro_rules! styled_tag_compile_tests {
    ($($name:ident => $tag:ident),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                let _el = rsx! { <$tag styled>{"content"}</$tag> };
            }
        )+
    };
}

macro_rules! flag_attr_compile_tests {
    ($($name:ident => $attr:ident),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                let _el = rsx! { <div $attr /> };
            }
        )+
    };
}

macro_rules! value_attr_compile_tests {
    ($($name:ident => $attr:ident = $value:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                let _el = rsx! { <div $attr={$value} /> };
            }
        )+
    };
}

mod ui {
    use super::MockElement;

    #[allow(non_snake_case)]
    pub fn TaskCard() -> MockElement {
        MockElement
    }

    pub struct TaskCardBuilder;

    impl TaskCardBuilder {
        pub fn new(_: u64) -> Self {
            Self
        }

        pub fn title(self, _: String) -> Self {
            self
        }

        pub fn compact(self) -> Self {
            self
        }
    }
}

#[path = "macro_cases/mod.rs"]
mod cases;
