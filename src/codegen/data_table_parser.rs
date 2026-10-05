use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use crate::parser::{RsxAttribute, RsxElement, RsxNode};
use crate::codegen::{
    attribute::{generate_attr_methods_with_mode, AttrHints},
    class::ClassMode,
    element::generate_node_checked,
};

fn as_element<'a>(node: &'a RsxNode, tag: &str) -> Option<&'a RsxElement> {
    let RsxNode::Element(el) = node else { return None; };
    (el.name.to_string() == tag).then_some(el)
}

fn get_string_literal(node: &RsxNode) -> Option<String> {
    let RsxNode::Expr(syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. })) = node else { return None; };
    Some(s.value())
}

fn get_attr_value<'a>(attrs: &'a [RsxAttribute], target: &str) -> Option<&'a syn::Expr> {
    attrs.iter().find_map(|attr| {
        let RsxAttribute::Value { name, value } = attr else { return None; };
        (name.to_string() == target).then_some(value)
    })
}

pub fn parse_data_table(element: &RsxElement) -> Result<TokenStream, TokenStream> {
    let mut columns = Vec::new();
    let mut group_headers = Vec::new();

    if let Some(thead_el) = element.children.iter().find_map(|c| as_element(c, "thead")) {
        let tr_rows: Vec<_> = thead_el.children.iter().filter_map(|c| as_element(c, "tr")).collect();

        if let Some((&main_tr, group_trs)) = tr_rows.split_last() {
            group_headers = group_trs.iter().map(|&group_tr| {
                let row_groups: Vec<_> = group_tr.children.iter().filter_map(|th_node| {
                    let th = as_element(th_node, "th")?;
                    let label = th.children.iter().find_map(get_string_literal).unwrap_or_default();
                    let span = get_attr_value(&th.attributes, "colspan")
                        .map_or_else(|| quote! { 1 }, |val| quote! { #val });

                    Some(quote! { gpui_kit::component::table::ColumnGroup::new(#label, #span) })
                }).collect();

                quote! { vec![ #( #row_groups ),* ] }
            }).collect();

            columns = main_tr.children.iter().filter_map(|th_node| {
                let th = as_element(th_node, "th")?;
                Some((|| -> Result<TokenStream, TokenStream> {
                    let id_attr = get_attr_value(&th.attributes, "id").ok_or_else(|| {
                        syn::Error::new(th.name.span(), "<th> inside <DataTable> must have an `id` attribute").to_compile_error()
                    })?;

                    let name_text = th.children.iter().find_map(get_string_literal).unwrap_or_default();
                    let col_methods = th.attributes.iter().filter_map(|attr| match attr {
                        RsxAttribute::Value { name, value } if name.to_string() != "id" => Some(quote! { .#name(#value) }),
                        RsxAttribute::Flag(name) => Some(quote! { .#name() }),
                        _ => None,
                    });

                    Ok(quote! {
                        gpui_kit::component::table::Column::new(#id_attr, #name_text)
                        #( #col_methods )*
                    })
                })())
            }).collect::<Result<Vec<_>, _>>()?;
        }
    }

    let tbody_el = element.children.iter().find_map(|c| as_element(c, "tbody"));
    let col_els: Vec<&RsxElement> = element.children.iter().filter_map(|c| as_element(c, "Col")).collect();

    let items = tbody_el
        .and_then(|el| get_attr_value(&el.attributes, "items"))
        .or_else(|| get_attr_value(&element.attributes, "rows"))
        .ok_or_else(|| syn::Error::new(element.name.span(), "<tbody> requires `items` attribute (or use <Col> columns with a `rows` attribute on <DataTable>)").to_compile_error())?;

    let items_expr = quote! {
        zopra::components::declarative_table::IntoRows::into_rows(#items)
    };

    let mut row_ident = quote! { _item };
    let mut tr_delegate_methods = Vec::new();
    let mut match_arms = Vec::new();
    let mut sorters: Vec<TokenStream> = Vec::new();

    if !col_els.is_empty() {
        let col = col_els[0];
        let accessor = get_attr_value(&col.attributes, "r")
            .ok_or_else(|| syn::Error::new(col.name.span(), "<Col> requires an `r` accessor: r={|row| row.field}").to_compile_error())?;
        if let syn::Expr::Closure(c) = accessor
            && let Some(arg) = c.inputs.first()
        {
            row_ident = quote! { #arg };
        }
    }

    for (ix, col) in col_els.iter().enumerate() {
        let accessor = get_attr_value(&col.attributes, "r")
            .ok_or_else(|| syn::Error::new(col.name.span(), "<Col> requires an `r` accessor: r={|row| row.field}").to_compile_error())?;
        let title = get_attr_value(&col.attributes, "title")
            .map(|e| quote! { #e })
            .unwrap_or_else(|| quote! { "" });
        let key = get_attr_value(&col.attributes, "id")
            .map(|e| quote! { #e })
            .unwrap_or_else(|| {
                let k = format!("col{ix}");
                quote! { #k }
            });

        let mut col_methods = Vec::new();
        let mut sortable = false;
        for attr in &col.attributes {
            match attr {
                RsxAttribute::Value { name, value } if name.to_string() != "r" && name.to_string() != "title" && name.to_string() != "id" => {
                    col_methods.push(quote! { .#name(#value) });
                }
                RsxAttribute::Flag(name) => {
                    if name.to_string() == "sortable" {
                        sortable = true;
                    }
                    col_methods.push(quote! { .#name() });
                }
                _ => {}
            }
        }

        columns.push(quote! {
            gpui_kit::component::table::Column::new(#key, #title)
                #( #col_methods )*
        });

        let cell = quote! { {
            let __zopra_cell_val = zopra::components::declarative_table::col_cell(#accessor, #row_ident);
            gpui_kit::component::table::TableCell::new().p_0().h_full().child(__zopra_cell_val)
        } };
        match_arms.push(quote! { #key => { #cell.into_any_element() } });

        let sorter_expr = if sortable {
            quote! { Some(zopra::components::declarative_table::col_sorter(#accessor, __zopra_rows.as_slice())) }
        } else {
            quote! { None }
        };
        sorters.push(sorter_expr);
    }

    if let Some(tbody_el) = tbody_el {
        let closure = tbody_el.children.iter().find_map(|c| {
            let RsxNode::Expr(syn::Expr::Closure(c)) = c else { return None; };
            Some(c)
        });

        if let Some(closure) = closure
            && let syn::Expr::Macro(expr_mac) = &*closure.body
            && expr_mac.mac.path.segments.last().is_some_and(|s| s.ident == "view")
        {
            if let Some(arg) = closure.inputs.first() {
                row_ident = quote! { #arg };
            }

            let tr_el = syn::parse2::<RsxElement>(expr_mac.mac.tokens.clone())
                .map_err(|e| syn::Error::new(expr_mac.span(), format!("Failed to parse inner view! TR: {}", e)).to_compile_error())?;

            if tr_el.name.to_string() == "tr" {
                // Restored: Common td methods hook
                let common_td_methods = Vec::<TokenStream>::new();

                let tr_methods = tr_el.attributes.iter().fold(Vec::new(), |mut methods, attr| {
                    if let RsxAttribute::Value { name, value } = attr
                        && matches!(name.to_string().as_str(), "on_context_menu" | "on_click" | "on_double_click")
                    {
                        tr_delegate_methods.push(quote! { .#name(move |_, _| { #value() }) });
                        return methods;
                    }

                    let name_str = match attr {
                        RsxAttribute::Value { name, .. } | RsxAttribute::Flag(name) => name.to_string(),
                        RsxAttribute::StateClass { method, .. } => method.to_string(),
                        RsxAttribute::When { .. } | RsxAttribute::WhenSome { .. } | RsxAttribute::WhenClass { .. } => String::new(),
                    };

                    let static_class = if name_str == "class"
                        && let RsxAttribute::Value { value: syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }), .. } = attr
                    {
                        Some(lit_str.value())
                    } else { None };

                    generate_attr_methods_with_mode(
                        attr,
                        AttrHints { name: Some(&name_str), static_class: static_class.as_deref() },
                        &mut methods,
                        ClassMode::Permissive,
                    );

                    methods
                });

                if !tr_methods.is_empty() {
                    tr_delegate_methods.push(quote! {
                        .render_row(move |row_ix, #row_ident, _window, _cx| {
                            use gpui_kit::IntoElement;
                            gpui_kit::div()
                                .id(row_ix)
                                #( #tr_methods )*
                                .child(gpui_kit::div().absolute().inset_0() #( #tr_methods )*)
                        })
                    });
                }

                match_arms = tr_el.children.iter().filter_map(|td_node| {
                    let td = as_element(td_node, "td")?;
                    Some((|| -> Result<TokenStream, TokenStream> {
                        let td_id = get_attr_value(&td.attributes, "id").ok_or_else(|| {
                            syn::Error::new(td.name.span(), "<td> inside <tbody items> must have an `id` attribute").to_compile_error()
                        })?;

                        let compiled_children = td.children.iter()
                            .map(|node| generate_node_checked(node, false, ClassMode::Permissive, &[]))
                            .collect::<Result<Vec<_>, _>>()?;

                        let td_methods = td.attributes.iter().fold(Vec::new(), |mut methods, attr| {
                            match attr {
                                RsxAttribute::Value { name, value } if name.to_string() != "id" => {
                                    let name_str = name.to_string();
                                    let static_class = if name_str == "class"
                                        && let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(lit_str), .. }) = value
                                    {
                                        Some(lit_str.value())
                                    } else { None };

                                    generate_attr_methods_with_mode(
                                        attr,
                                        AttrHints { name: Some(&name_str), static_class: static_class.as_deref() },
                                        &mut methods,
                                        ClassMode::Permissive
                                    );
                                }
                                RsxAttribute::Flag(name) => methods.push(quote! { .#name() }),
                                _ => {}
                            }
                            methods
                        });

                        // Restored: #( #common_td_methods )* interpolation
                        let cell_content = if compiled_children.len() == 1 {
                            let child = &compiled_children[0];
                            quote! { gpui_kit::component::table::TableCell::new().p_0().h_full() #( #common_td_methods )* #( #td_methods )* .child(#child) }
                        } else {
                            quote! { gpui_kit::component::table::TableCell::new() #( #common_td_methods )* #( #td_methods )* #( .child(#compiled_children) )* }
                        };

                        Ok(quote! { #td_id => { #cell_content.into_any_element() } })
                    })())
                }).collect::<Result<Vec<_>, _>>()?;
            }
        }
    }

    let delegate_attrs = ["on_sort", "on_context_menu", "on_lazy_load", "group_headers"];
    let state_attrs = ["row_selectable", "cell_selectable", "col_selectable", "loop_selection", "row_header", "sortable"];
    let skip_attrs: &[&str] = if tbody_el.is_some() { &[] } else { &["rows"] };

    let (delegate_methods, state_methods, table_methods, event_handlers) = element.attributes.iter().fold(
        (tr_delegate_methods, Vec::new(), Vec::new(), Vec::new()),
        |(mut del, mut state, mut tab, mut events), attr| {
            match attr {
                RsxAttribute::Value { name, value } => {
                    let name_str = name.to_string();
                    if skip_attrs.contains(&name_str.as_str()) {
                    } else if delegate_attrs.contains(&name_str.as_str()) {
                        del.push(quote! { .#name(#value) });
                    } else if state_attrs.contains(&name_str.as_str()) {
                        state.push(quote! { .#name(#value) });
                    } else {
                        let mapped_event = match name_str.as_str() {
                            "on_select_row" => Some(quote! {
                                let __zopra_on_select_row: Box<dyn Fn(usize, &mut gpui_kit::App)> = Box::new(#value);
                                zopra::hooks::use_event(&_state, move |event, cx| {
                                    if let gpui_kit::component::table::TableEvent::SelectRow(ix) = event {
                                        __zopra_on_select_row(*ix, cx);
                                    }
                                }, cx);
                            }),
                            "on_double_click_row" => Some(quote! { .on_double_click_row(#value) }),
                            "on_right_click_row" => Some(quote! { .on_right_click_row(#value) }),
                            "on_select_cell" => Some(quote! { .on_select_cell(#value) }),
                            "on_double_click_cell" => Some(quote! { .on_double_click_cell(#value) }),
                            "on_right_click_cell" => Some(quote! { .on_right_click_cell(#value) }),
                            "on_select_column" => Some(quote! { .on_select_column(#value) }),
                            "on_move_column" => Some(quote! { .on_move_column(#value) }),
                            "on_column_widths_changed" => Some(quote! { .on_column_widths_changed(#value) }),
                            _ => None,
                        };
                        if name_str.as_str() == "on_select_row" {
                            events.push(mapped_event.unwrap());
                        } else if let Some(event) = mapped_event {
                            tab.push(event);
                        } else {
                            tab.push(quote! { .#name(#value) });
                        }
                    }
                },
                RsxAttribute::Flag(name) => {
                    let name_str = name.to_string();
                    if delegate_attrs.contains(&name_str.as_str()) {
                        del.push(quote! { .#name(true) });
                    } else if state_attrs.contains(&name_str.as_str()) {
                        state.push(quote! { .#name(true) });
                    } else {
                        tab.push(quote! { .#name(true) });
                    }
                },
                _ => {}
            }
            (del, state, tab, events)
        }
    );

    let group_headers_method = (!group_headers.is_empty())
        .then(|| quote! { .group_headers(vec![ #( #group_headers ),* ]) });

    let sorters_method = (!sorters.is_empty())
        .then(|| quote! { .sorters(vec![ #( #sorters ),* ]) });

    // 4. Output final AST
    Ok(quote! {
        {
            let __zopra_rows = #items_expr;
            let _state = zopra::hooks::use_table_with(
                || {
                    let _delegate = zopra::components::declarative_table::DeclarativeTableDelegate::new(
                        __zopra_rows.clone(),
                        vec![ #( #columns ),* ],
                        |#row_ident, col_id, _window, _cx| {
                            use gpui_kit::IntoElement;
                            match col_id {
                                #( #match_arms )*
                                _ => gpui_kit::component::table::TableCell::new().into_any_element(),
                            }
                        }
                    )
                    #group_headers_method
                    #sorters_method
                    #( #delegate_methods )*;
                    _delegate
                },
                |state| state #( #state_methods )*,
                window,
                cx
            );

            _state.update(cx, |state, cx| {
                if state.delegate_mut().update_data(&__zopra_rows) {
                    cx.notify();
                }
            });

            #( #event_handlers )*

            gpui_kit::component::table::DataTable::new(&_state)
            #( #table_methods )*
        }
    })
}
