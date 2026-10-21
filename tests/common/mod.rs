#![allow(dead_code)]

//! 共享 Mock GPUI 类型
//!
//! 所有测试文件共用的 MockElement 和辅助函数，
//! 避免在 macro_tests.rs 和 coverage_tests.rs 中重复定义。

mod capture;
mod constructors;
mod element;
mod styled;
mod types;

pub use capture::*;
#[allow(unused_imports)]
pub use constructors::*;
pub use types::*;
