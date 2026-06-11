use std::cell::RefCell;

// 捕获最近一次自动生成的 ID（以 `__rsx_` 开头）。
// 供 auto-ID 格式验证测试使用。
thread_local! {
    pub static LAST_AUTO_ID: RefCell<Option<String>> = const { RefCell::new(None) };
    pub static RGB_CALLS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    pub static RGBA_CALLS: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    pub static BORDER_CALLS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    pub static FONT_WEIGHT_CALLS: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    pub static LENGTH_CALLS: RefCell<Vec<(&'static str, f32)>> = const { RefCell::new(Vec::new()) };
    pub static INTEGER_CALLS: RefCell<Vec<(&'static str, i32)>> = const { RefCell::new(Vec::new()) };
}

pub fn take_border_calls() -> Vec<&'static str> {
    BORDER_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回最近捕获的 auto-ID，并清空缓存。
pub fn take_last_auto_id() -> Option<String> {
    LAST_AUTO_ID.with(|c| c.borrow_mut().take())
}

/// 返回测试期间捕获的 rgb() 入参，并清空缓存。
pub fn take_rgb_calls() -> Vec<u32> {
    RGB_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的 rgba() 入参，并清空缓存。
pub fn take_rgba_calls() -> Vec<u32> {
    RGBA_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的 font_weight() 入参，并清空缓存。
pub fn take_font_weight_calls() -> Vec<f32> {
    FONT_WEIGHT_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的长度 helper 入参，并清空缓存。
pub fn take_length_calls() -> Vec<(&'static str, f32)> {
    LENGTH_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}

/// 返回测试期间捕获的整数 helper 入参，并清空缓存。
pub fn take_integer_calls() -> Vec<(&'static str, i32)> {
    INTEGER_CALLS.with(|c| c.borrow_mut().drain(..).collect())
}
