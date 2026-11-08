use serde_json::{Value, json};
use std::path::Path;

pub fn inspect(
    metadata: &Value,
    sources: &[(impl AsRef<Path>, String)],
    dx: Option<(&str, bool)>,
) -> Value {
    let identities = crate::doctor::inspect(metadata);
    let engines = identities["engines"].as_array().unwrap();
    let enabled = !engines.is_empty()
        && engines.iter().all(|engine| {
            metadata["resolve"]["nodes"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|node| {
                    node["id"] == engine["id"]
                        && node["features"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .any(|feature| feature == "hot-reload")
                })
        });
    let collisions = sources
        .iter()
        .flat_map(|(path, source)| {
            gpui_rsx_tools::macro_invocations(source)
                .unwrap_or_default()
                .into_iter()
                .filter(|call| call.name == "rsx")
                .map(|call| json!({"path": path.as_ref(), "start": call.start, "end": call.end}))
        })
        .collect::<Vec<_>>();
    json!({
        "ok": enabled && dx.is_some_and(|(_, hot_patch)| hot_patch) && identities["ok"] == true,
        "runtimeFeatureEnabled": enabled,
        "dioxusCli": dx.map(|(version, _)| version),
        "templateNameCollisions": collisions,
        "checks": [
            {"code": "hot-reload/runtime", "status": if enabled {"passed"} else {"failed"},
             "message": "The selected GPUI backend must enable hot-reload. Resolve the application's hotreload feature with doctor --features hotreload."},
            {"code": "hot-reload/cli", "status": if dx.is_some_and(|(_, hot_patch)| hot_patch) {"passed"} else {"failed"},
             "message": "The installed dx serve must support --hot-patch. The reproduction pins Dioxus CLI 0.7.10; patch behavior on other versions needs its own verification."},
            {"code": "hot-reload/macro-name", "status": if collisions.is_empty() {"passed"} else {"warning"},
             "message": "Dioxus CLI 0.7.10 intercepts macros named rsx, including qualified paths. Alias GPUI-RSX as gpui_view or view. Source checks include inactive cfg branches."},
            {"code": "hot-reload/patch", "status": "unverified",
             "message": "Readiness checks do not observe patches. With --hot-patch --hot-reload true, verify a rendered revision change, the same PID, and preserved timer/click state using tests/fixtures/hot-reload."}
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(enabled: bool) -> Value {
        json!({
            "workspace_members": ["app"],
            "packages": [
                {"id":"app", "name":"app"},
                {"id":"gpui", "name":"gpui-pre", "version":"0.3.11"}
            ],
            "resolve": {"nodes": [
                {"id":"app", "deps":[{"pkg":"gpui"}]},
                {"id":"gpui", "deps":[], "features": if enabled {vec!["hot-reload"]} else {vec![]}}
            ]}
        })
    }

    #[test]
    fn readiness_requires_the_runtime_feature_and_cli() {
        let sources: Vec<(&Path, String)> = Vec::new();
        assert_eq!(
            inspect(&metadata(true), &sources, Some(("dx 0.7.10", true)))["ok"],
            true
        );
        assert_eq!(
            inspect(&metadata(false), &sources, Some(("dx 0.7.10", true)))["ok"],
            false
        );
        assert_eq!(
            inspect(&metadata(true), &sources, Some(("dx 0.6.3", false)))["ok"],
            false
        );
        assert_eq!(inspect(&metadata(true), &sources, None)["ok"], false);
    }

    #[test]
    fn qualified_macros_are_reported_but_aliases_are_not() {
        let sources = [(
            Path::new("src/main.rs"),
            "use gpui_rsx::rsx as gpui_view; fn main() { gpui_rsx::rsx! { <div/> }; gpui_view! { <div/> }; }".into(),
        )];
        let report = inspect(&metadata(true), &sources, Some(("dx 0.7.10", true)));
        assert_eq!(
            report["templateNameCollisions"].as_array().unwrap().len(),
            1
        );
        assert_eq!(report["checks"][3]["status"], "unverified");
    }

    #[test]
    fn a_macro_renamed_to_rsx_still_collides() {
        let sources = [(
            Path::new("src/main.rs"),
            "use gpui_rsx::rsx_tw as rsx; fn render(){rsx!{<div/>};}".into(),
        )];
        let report = inspect(&metadata(true), &sources, Some(("dx 0.7.10", true)));
        assert_eq!(
            report["templateNameCollisions"].as_array().unwrap().len(),
            1
        );
    }
}
