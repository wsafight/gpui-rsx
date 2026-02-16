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
//!                     bg="blue-500"
//!                     text="white"
//!                     px="4"
//!                     py="2"
//!                     rounded="md"
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
mod parser;

use codegen::generate_code;
use parser::RsxElement;

/// RSX 宏 - 将 HTML-like 语法转换为 GPUI 代码
///
/// # 语法
///
/// ## 基本元素
/// ```ignore
/// rsx! { <div>{"Hello"}</div> }
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
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let element = parse_macro_input!(input as RsxElement);
    let code = generate_code(&element);
    TokenStream::from(code)
}
