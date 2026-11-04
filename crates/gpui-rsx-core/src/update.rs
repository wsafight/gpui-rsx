use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Token, parse::Parse, parse::ParseStream};

struct UpdateInput {
    view: Expr,
    context: Expr,
    update: Expr,
}

impl Parse for UpdateInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let view = input.parse()?;
        input.parse::<Token![,]>()?;
        let context = input.parse()?;
        input.parse::<Token![,]>()?;
        let update = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        Ok(Self {
            view,
            context,
            update,
        })
    }
}

/// Expands an explicit synchronous state update followed by a view notification.
pub fn expand_update_with_path(input: TokenStream, gpui: &syn::Path) -> TokenStream {
    let source = input.to_string();
    let mut name = String::from("__gpui_rsx_update");
    while source.contains(&name) {
        name.push('_');
    }
    let helper = syn::Ident::new(&name, proc_macro2::Span::mixed_site());
    let UpdateInput {
        view,
        context,
        update,
    } = match syn::parse2(input) {
        Ok(input) => input,
        Err(error) => return error.to_compile_error(),
    };
    quote! {
        {
            fn #helper<View: 'static, Output>(
                view: &mut View,
                cx: &mut #gpui::Context<'_, View>,
                update: impl ::core::ops::FnOnce(&mut View, &mut #gpui::Context<'_, View>) -> Output,
            ) -> Output {
                let result = update(view, cx);
                cx.notify();
                result
            }
            #helper(#view, #context, #update)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_exactly_three_arguments_and_accepts_a_trailing_comma() {
        let path = syn::parse_quote!(::gpui);
        for input in [quote!(), quote!(view, cx), quote!(view, cx, update, extra)] {
            assert!(
                expand_update_with_path(input, &path)
                    .to_string()
                    .contains("compile_error")
            );
        }
        let output =
            expand_update_with_path(quote!(view(), context(), updater(),), &path).to_string();
        assert_eq!(output.matches("view ()").count(), 1);
        assert_eq!(output.matches("context ()").count(), 1);
        assert_eq!(output.matches("updater ()").count(), 1);
    }
}
