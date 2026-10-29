//! In-memory theme construction for tools and tests.
use super::*;

/// Builder for in-memory [`Theme`] values used by tools and integrations.
#[derive(Clone, Debug, Default)]
pub struct ThemeBuilder {
    theme: Theme,
}

impl ThemeBuilder {
    pub fn scale(mut self, scale: SpacingScale) -> Self {
        self.theme.scale = Some(scale);
        self
    }

    pub fn color(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.colors.insert(name.into(), value);
        self
    }

    pub fn radius(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.radius.insert(name.into(), value);
        self
    }

    pub fn spacing(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.spacing.insert(name.into(), value);
        self
    }

    pub fn text_size(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.text_size.insert(name.into(), value);
        self
    }

    pub fn line_height(mut self, name: impl Into<String>, value: TokenValue) -> Self {
        self.theme.line_height.insert(name.into(), value);
        self
    }

    pub fn opacity(mut self, name: impl Into<String>, value: f32) -> Self {
        self.theme
            .opacity
            .insert(name.into(), TokenValue::Number(value));
        self
    }

    pub fn shadow(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.theme
            .shadow
            .insert(name.into(), TokenValue::Expr(value.into()));
        self
    }

    pub fn build(self) -> Theme {
        self.theme
    }

    /// Builds a theme after validating token names, types and expression syntax.
    pub fn try_build(self) -> Result<Theme, String> {
        self.theme.validate()?;
        Ok(self.theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_every_token_section_and_scale() {
        let theme = Theme::builder()
            .scale(SpacingScale::Tailwind)
            .color(
                "primary",
                TokenValue::Color {
                    hex: 0x123456,
                    alpha: false,
                },
            )
            .radius("card", TokenValue::Length("px(12.0)".into()))
            .spacing("gutter", TokenValue::Length("px(18.0)".into()))
            .text_size("body", TokenValue::Length("px(14.0)".into()))
            .line_height("body", TokenValue::Length("px(20.0)".into()))
            .opacity("muted", 0.6)
            .shadow("card", "cx.theme().shadow")
            .build();
        assert_eq!(theme.scale(), Some(SpacingScale::Tailwind));
        assert!(theme.colors().contains_key("primary"));
        assert!(theme.radius().contains_key("card"));
        assert!(theme.spacing().contains_key("gutter"));
        assert!(theme.text_size().contains_key("body"));
        assert!(theme.line_height().contains_key("body"));
        assert_eq!(theme.opacity().get("muted"), Some(&TokenValue::Number(0.6)));
        assert_eq!(
            theme.shadow().get("card"),
            Some(&TokenValue::Expr("cx.theme().shadow".into()))
        );
        assert_eq!(theme.tokens().len(), 7);
    }

    #[test]
    fn an_empty_builder_matches_a_new_theme() {
        assert_eq!(Theme::builder().build(), Theme::new());
        assert!(Theme::builder().build().scale().is_none());
    }

    #[test]
    fn try_build_accepts_valid_tokens() {
        let theme = Theme::builder()
            .color(
                "brand",
                TokenValue::Color {
                    hex: 0xabcdef,
                    alpha: true,
                },
            )
            .spacing("gutter", TokenValue::Length("rems(1.5)".into()))
            .opacity("muted", 0.5)
            .shadow("card", "cx.theme().shadow")
            .try_build()
            .unwrap();
        assert_eq!(theme.tokens().len(), 4);
    }

    #[test]
    fn try_build_rejects_invalid_names_types_and_expressions() {
        for theme in [
            Theme::builder()
                .color(
                    "",
                    TokenValue::Color {
                        hex: 0,
                        alpha: false,
                    },
                )
                .build(),
            Theme::builder()
                .color(
                    "bad name",
                    TokenValue::Color {
                        hex: 0,
                        alpha: false,
                    },
                )
                .build(),
            Theme::builder()
                .color(
                    "brand",
                    TokenValue::Color {
                        hex: 0x1ffffff,
                        alpha: false,
                    },
                )
                .build(),
            Theme::builder()
                .color("brand", TokenValue::Length("px(1.0)".into()))
                .build(),
            Theme::builder()
                .radius("card", TokenValue::Number(1.0))
                .build(),
            Theme::builder().opacity("muted", f32::NAN).build(),
            Theme::builder().opacity("muted", 1.5).build(),
            Theme::builder().shadow("card", "not (valid").build(),
        ] {
            assert!(theme.validate().is_err());
        }
    }

    #[test]
    fn try_build_rejects_size_named_color_conflicts() {
        let theme = Theme::builder()
            .color(
                "size-body",
                TokenValue::Color {
                    hex: 0,
                    alpha: false,
                },
            )
            .text_size("body", TokenValue::Length("px(14.0)".into()))
            .try_build();
        assert!(theme.unwrap_err().contains("conflicts"));
    }
}
