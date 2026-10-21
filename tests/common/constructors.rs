use super::*;

// 模拟 GPUI 构造函数
#[allow(dead_code)]
pub fn div() -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn svg() -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn img<T>(_source: T) -> MockElement {
    MockElement
}
#[allow(dead_code)]
pub fn canvas<P, Paint, T>(_prepaint: P, _paint: Paint) -> MockElement
where
    P: FnOnce(Bounds, &mut Window, &mut App) -> T,
    Paint: FnOnce(Bounds, T, &mut Window, &mut App),
{
    MockElement
}
pub fn rgb(hex: u32) -> u32 {
    RGB_CALLS.with(|c| c.borrow_mut().push(hex));
    hex
}
pub fn rgba(hex: u32) -> u32 {
    RGBA_CALLS.with(|c| c.borrow_mut().push(hex));
    hex
}
pub fn px(val: f32) -> f32 {
    LENGTH_CALLS.with(|c| c.borrow_mut().push(("px", val)));
    val
}
pub fn rems(val: f32) -> f32 {
    LENGTH_CALLS.with(|c| c.borrow_mut().push(("rems", val)));
    val
}
pub fn relative(val: f32) -> f32 {
    LENGTH_CALLS.with(|c| c.borrow_mut().push(("relative", val)));
    val
}
pub fn auto() -> f32 {
    0.0
}

// 模拟自定义组件构造函数
#[allow(non_snake_case, dead_code)]
pub fn MyComponent() -> MockElement {
    MockElement
}
#[allow(non_snake_case, dead_code)]
pub fn CustomWidget() -> MockElement {
    MockElement
}
