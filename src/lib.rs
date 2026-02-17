//! # GPUI-RSX
//!
//! 一个为 GPUI 提供 JSX-like 语法的过程宏，让 UI 开发更加简洁和直观。
//!
//! ## 示例
//!
//! ```rust,ignore
//! use gpui::*;
//! use gpui_rsx::rsx;
//!
//! impl Render for MyView {
//!     fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
//!         rsx! {
//!             <div class="flex flex-col gap-4 p-4">
//!                 <h1>{"Hello GPUI"}</h1>
//!                 <button
//!                     bg={rgb(0x3b82f6)}
//!                     text_color={rgb(0xffffff)}
//!                     px_4
//!                     py_2
//!                     rounded_md
//!                     onClick={cx.listener(|view, _, cx| {
//!                         println!("clicked");
//!                     })}
//!                 >
//!                     {"Click me"}
//!                 </button>
//!             </div>
//!         }
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod codegen;
mod diagnostics;
mod parser;

use codegen::generate_body;
use parser::RsxBody;

/// RSX 宏 - 将 HTML-like 语法转换为 GPUI 代码
///
/// # 语法
///
/// ## 基本元素
/// ```ignore
/// rsx! { <div>{"Hello"}</div> }
/// ```
///
/// ## Fragment（多根节点）
/// ```ignore
/// rsx! {
///     <>
///         <div>{"First"}</div>
///         <div>{"Second"}</div>
///     </>
/// }
/// ```
///
/// ## 属性
/// ```ignore
/// rsx! {
///     <div
///         flex
///         flex_col
///         gap={px(16.0)}
///         bg={rgb(0xffffff)}
///     />
/// }
/// ```
///
/// ## 嵌套
/// ```ignore
/// rsx! {
///     <div>
///         <span>{"Item 1"}</span>
///         <span>{"Item 2"}</span>
///     </div>
/// }
/// ```
///
/// ## 表达式
/// ```ignore
/// rsx! {
///     <div>
///         {format!("Count: {}", self.count)}
///         {self.render_child()}
///     </div>
/// }
/// ```
///
/// ## 条件渲染
/// ```ignore
/// rsx! {
///     <div>
///         {if self.show {
///             rsx! { <span>{"Visible"}</span> }
///         } else {
///             rsx! { <span>{"Hidden"}</span> }
///         }}
///     </div>
/// }
/// ```
///
/// ## For 循环
/// ```ignore
/// rsx! {
///     <ul>
///         {for item in &self.items {
///             <li>{item.clone()}</li>
///         }}
///     </ul>
/// }
/// ```
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let body = parse_macro_input!(input as RsxBody);
    let code = generate_body(&body);
    TokenStream::from(code)
}
