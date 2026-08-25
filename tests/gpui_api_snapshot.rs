use std::collections::BTreeSet;
use std::path::PathBuf;

const FIXTURE: &str = include_str!("fixtures/gpui_stateful_methods_e973593.txt");

#[test]
fn resolved_gpui_stateful_api_matches_snapshot() {
    let Some(source_root) = std::env::var_os("GPUI_SOURCE_ROOT") else {
        eprintln!("skipping live GPUI API check; GPUI_SOURCE_ROOT is not set");
        return;
    };

    let source_file = PathBuf::from(source_root).join("src/elements/div.rs");
    let source = std::fs::read_to_string(&source_file)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_file.display()));
    let syntax = syn::parse_file(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", source_file.display()));

    let actual = syntax
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Trait(item) if item.ident == "StatefulInteractiveElement" => Some(
                item.items
                    .iter()
                    .filter_map(|item| match item {
                        syn::TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>(),
            ),
            _ => None,
        })
        .expect("StatefulInteractiveElement trait not found");
    let expected = FIXTURE
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected, "resolved GPUI stateful API drifted");
}
