//! Dynamic color parsing shared by inline and crate-wide helpers.
use crate::codegen::tables::{COLOR_FAMILIES, COLOR_SHADES, lookup_color};
use proc_macro2::TokenStream;
use quote::quote;

/// 生成动态颜色解析回退代码。
///
/// 相比为 text/bg/border 三个前缀展开完整色板 match arm，这里只在动态路径中
/// 解析 `prefix-family-shade`，把展开体积从 700+ 个颜色分支降到 22 个色系分支。
pub(super) fn generate_color_fallback_code() -> TokenStream {
    let shade_arms = COLOR_SHADES.iter().enumerate().map(|(idx, shade)| {
        quote! { #shade => #idx, }
    });

    let family_arms = COLOR_FAMILIES.iter().map(|family| {
        let values = COLOR_SHADES.iter().map(|shade| {
            let key = format!("{family}_{shade}");
            lookup_color(&key).expect("COLOR_FAMILIES/COLOR_SHADES must match lookup_color")
        });
        quote! {
            #family => {
                const VALUES: [u32; 11] = [#(#values),*];
                Some(VALUES[shade_index])
            }
        }
    });

    quote! {
        fn __rsx_hex_digit(byte: u8) -> Option<u32> {
            match byte {
                b'0'..=b'9' => Some((byte - b'0') as u32),
                b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
                b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
                _ => None,
            }
        }

        fn __rsx_parse_hex_color(color: &str) -> Option<u32> {
            let inner = color.strip_prefix("[#")?.strip_suffix(']')?;
            let bytes = inner.as_bytes();
            // `from_str_radix` alone would also accept a leading `+`.
            if !bytes.iter().all(u8::is_ascii_hexdigit) {
                return None;
            }
            match bytes.len() {
                8 => u32::from_str_radix(inner, 16).ok(),
                6 => u32::from_str_radix(inner, 16).ok(),
                4 => {
                    let r = __rsx_hex_digit(bytes[0])?;
                    let g = __rsx_hex_digit(bytes[1])?;
                    let b = __rsx_hex_digit(bytes[2])?;
                    let a = __rsx_hex_digit(bytes[3])?;
                    let expand = |n: u32| (n << 4) | n;
                    Some(expand(r) << 24 | expand(g) << 16 | expand(b) << 8 | expand(a))
                }
                3 => {
                    let r = __rsx_hex_digit(bytes[0])?;
                    let g = __rsx_hex_digit(bytes[1])?;
                    let b = __rsx_hex_digit(bytes[2])?;
                    Some(r << 20 | r << 16 | g << 12 | g << 8 | b << 4 | b)
                }
                _ => None,
            }
        }

        fn __rsx_parse_u8_component(raw: &str) -> Option<u8> {
            raw.trim().parse::<u8>().ok()
        }

        fn __rsx_parse_alpha_component(raw: &str) -> Option<u8> {
            let value = raw.trim().parse::<f32>().ok()?;
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return None;
            }
            Some((value * 255.0).round() as u8)
        }

        fn __rsx_parse_color_function(color: &str) -> Option<(u32, bool)> {
            let inner = color.strip_prefix('[')?.strip_suffix(']')?;

            if let Some(args) = inner.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
                let mut parts = args.split(',');
                let r = __rsx_parse_u8_component(parts.next()?)?;
                let g = __rsx_parse_u8_component(parts.next()?)?;
                let b = __rsx_parse_u8_component(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                return Some((((r as u32) << 16) | ((g as u32) << 8) | b as u32, false));
            }

            if let Some(args) = inner.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
                let mut parts = args.split(',');
                let r = __rsx_parse_u8_component(parts.next()?)?;
                let g = __rsx_parse_u8_component(parts.next()?)?;
                let b = __rsx_parse_u8_component(parts.next()?)?;
                let a = __rsx_parse_alpha_component(parts.next()?)?;
                if parts.next().is_some() {
                    return None;
                }
                return Some((
                    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32,
                    true,
                ));
            }

            None
        }

        fn __rsx_parse_named_color(color: &str) -> Option<u32> {
            if color == "black" {
                return Some(0x000000);
            }
            if color == "white" {
                return Some(0xffffff);
            }

            let (family, shade) = color.rsplit_once('-')?;
            let shade_index = match shade {
                #(#shade_arms)*
                _ => return None,
            };

            match family {
                #(#family_arms,)*
                _ => None,
            }
        }

        fn __rsx_parse_color(color: &str) -> Option<(u32, bool)> {
            if let Some((base, raw)) = color.rsplit_once('/') {
                if base.contains('/') { return None; }
                let alpha = raw.parse::<f32>().ok()?;
                if !alpha.is_finite() || !(0.0..=100.0).contains(&alpha) { return None; }
                let (hex, rgba) = __rsx_parse_color(base)?;
                let rgb = if rgba { hex & 0xffffff00 } else { hex << 8 };
                return Some((rgb | (alpha / 100.0 * 255.0).round() as u32, true));
            }
            if let Some(hex) = __rsx_parse_hex_color(color) {
                let hex_inner = color.strip_prefix("[#")?.strip_suffix(']')?;
                let is_rgba = hex_inner.len() == 8 || hex_inner.len() == 4;
                return Some((hex, is_rgba));
            }
            __rsx_parse_color_function(color)
                .or_else(|| __rsx_parse_named_color(color).map(|color| (color, false)))
        }

        // 生成代码不使用 let-chains：调用方 crate 可能仍是 edition 2021。
        if let Some(rest) = class.strip_prefix("text-") {
            if let Some((color, is_rgba)) = __rsx_parse_color(rest) {
                if is_rgba {
                    return el.text_color(rgba(color));
                }
                return el.text_color(rgb(color));
            }
        }
        if let Some(rest) = class.strip_prefix("bg-") {
            if let Some((color, is_rgba)) = __rsx_parse_color(rest) {
                if is_rgba {
                    return el.bg(rgba(color));
                }
                return el.bg(rgb(color));
            }
        }
        if let Some(rest) = class.strip_prefix("border-") {
            if let Some((color, is_rgba)) = __rsx_parse_color(rest) {
                if is_rgba {
                    return el.border_color(rgba(color));
                }
                return el.border_color(rgb(color));
            }
        }
    }
}
