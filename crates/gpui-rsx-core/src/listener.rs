use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Ident, Result, Token, parse::Parse, parse::ParseStream};

struct ListenerInput {
    context: Expr,
    notify: bool,
    handler: Expr,
}

impl Parse for ListenerInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let context = input.parse()?;
        input.parse::<Token![,]>()?;

        let second: Expr = input.parse()?;
        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }
        let (notify, handler) = if input.is_empty() {
            (false, second)
        } else if is_notify_marker(&second) {
            let handler = input.parse()?;
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            (true, handler)
        } else {
            return Err(syn::Error::new_spanned(
                second,
                "expected `notify` before the third argument",
            ));
        };
        if !input.is_empty() {
            return Err(input.error("expected a context, an optional `notify`, and a handler"));
        }
        Ok(Self {
            context,
            notify,
            handler,
        })
    }
}

fn is_notify_marker(expr: &Expr) -> bool {
    matches!(expr, Expr::Path(path) if path.path.is_ident("notify"))
}

/// Expands `rsx_listener!`, returning parse errors as `compile_error!` tokens.
pub fn expand_listener(input: TokenStream) -> TokenStream {
    expand_listener_with_path(input, &syn::parse_quote!(::gpui))
}

/// Expands a listener using the given GPUI path.
pub fn expand_listener_with_path(input: TokenStream, gpui: &syn::Path) -> TokenStream {
    match syn::parse2::<ListenerInput>(input) {
        Ok(input) => {
            let ListenerInput {
                context,
                notify,
                handler,
            } = input;
            if notify {
                let context_binding =
                    Ident::new("__gpui_rsx_context", proc_macro2::Span::mixed_site());
                quote! {
                    {
                        let #context_binding = &(#context);
                        ({
                            fn __gpui_rsx_listener_with_notify<View: 'static, Event: ?Sized>(
                                context: &#gpui::Context<'_, View>,
                                handler: impl ::core::ops::Fn(
                                    &mut View,
                                    &Event,
                                    &mut #gpui::Window,
                                    &mut #gpui::Context<'_, View>,
                                ) + 'static,
                            ) -> impl ::core::ops::Fn(&Event, &mut #gpui::Window, &mut #gpui::App) + 'static {
                                context.listener(move |view, event, window, cx| {
                                    handler(view, event, window, cx);
                                    cx.notify();
                                })
                            }
                            __gpui_rsx_listener_with_notify
                        })(#context_binding, #handler)
                    }
                }
            } else {
                quote! { (#context).listener(#handler) }
            }
        }
        Err(error) => error.to_compile_error(),
    }
}

#[cfg(test)]
mod tests {
    use super::expand_listener;
    use quote::quote;

    #[test]
    fn plain_listener_delegates_to_context() {
        let output = expand_listener(quote!(cx, handler)).to_string();
        assert_eq!(output, "(cx) . listener (handler)");
    }

    #[test]
    fn notify_listener_wraps_the_existing_callback_shape() {
        let output = expand_listener(quote!(cx, notify, |view, event, window, cx| {
            let _ = (view, event, window, cx);
        }))
        .to_string();
        assert!(output.contains("cx . notify ()"));
        assert!(output.contains("handler (view , event , window , cx)"));
    }

    #[test]
    fn context_and_handler_are_each_evaluated_once() {
        let output = expand_listener(quote!(make_context(), notify, make_handler())).to_string();
        assert_eq!(output.matches("make_context").count(), 1);
        assert_eq!(output.matches("make_handler").count(), 1);
    }

    #[test]
    fn a_lone_notify_is_a_handler_not_the_marker() {
        for input in [quote!(cx, notify), quote!(cx, notify,)] {
            assert_eq!(
                expand_listener(input).to_string(),
                "(cx) . listener (notify)"
            );
        }
    }

    #[test]
    fn handler_expressions_and_trailing_commas_are_supported() {
        for input in [
            quote!(cx, notify),
            quote!(cx, notify,),
            quote!(cx, Self::handle,),
            quote!(cx, notify, { move |_, _, _, _| {} },),
        ] {
            assert!(!expand_listener(input).to_string().contains("compile_error"));
        }
    }

    #[test]
    fn malformed_invocations_are_rejected() {
        for input in [
            quote!(),
            quote!(cx),
            quote!(cx, notify, ,),
            quote!(cx, notify handler),
            quote!(cx, handler, extra),
        ] {
            assert!(expand_listener(input).to_string().contains("compile_error"));
        }
    }
}
