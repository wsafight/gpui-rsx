//! 代码生成器
//!
//! 将解析后的 RSX 转换为 GPUI 代码
//!
//! 生成惯用的 GPUI 方法链模式：
//! ```ignore
//! div().id("auto_0").flex().bg(rgb(0xff)).on_click(handler).child("text")
//! ```
//!
//! # 模块结构
//!
//! - `tables`: 静态查找表（颜色、事件、属性映射等）
//! - `class`: CSS class 字符串解析，含统一的颜色处理
//! - `attribute`: RSX 属性到 GPUI 方法的转换
//! - `element`: 元素和子节点的代码生成

pub(crate) mod attribute;
pub(crate) mod class;
pub(crate) mod element;
pub(crate) mod tables;

// 重新导出公共 API
pub use element::generate_body;
