//! 代码生成器
//!
//! 将解析后的 RSX 转换为 GPUI 代码
//!
//! 生成惯用的 GPUI 方法链模式：
//! ```text
//! div().id("auto_0").flex().bg(rgb(0xff)).on_click(handler).child("text")
//! ```
//!
//! # 模块结构
//!
//! - `tables_generated`: 由 `cargo xtask gen-tables` 从 GPUI 源码生成的方法事实与色板
//! - `tables`: 手写语义层（属性别名、class 别名、长度前缀、标签默认样式）
//! - `class`: CSS class 字符串解析，含统一的颜色处理
//! - `attribute`: RSX 属性到 GPUI 方法的转换
//! - `element`: 元素和子节点的代码生成

pub mod attribute;
mod auto_id;
pub mod class;
pub mod element;
mod format_class;
mod key;
pub mod runtime;
pub mod tables;
#[rustfmt::skip]
pub mod tables_generated;
