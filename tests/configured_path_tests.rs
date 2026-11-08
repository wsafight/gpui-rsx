use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

/// A throwaway fixture crate. `tag` names its own target subdirectory: the tests in this
/// file run in parallel and every fixture shares one package name, so a shared target
/// directory lets concurrent `cargo check` runs clobber each other's build scripts.
struct Project {
    root: PathBuf,
    tag: &'static str,
}

impl Project {
    fn new(tag: &'static str) -> Self {
        static NEXT_PROJECT: AtomicU64 = AtomicU64::new(0);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = loop {
            let candidate = std::env::temp_dir().join(format!(
                "gpui-rsx-path-rebuild-{tag}-{}-{}",
                std::process::id(),
                NEXT_PROJECT.fetch_add(1, Ordering::Relaxed),
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture directory {}: {error}", candidate.display()),
            }
        };
        let project = Self {
            root: project_root,
            tag,
        };
        fs::create_dir(project.root.join("src")).unwrap();
        fs::write(
            project.root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"path-rebuild-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\
                 [dependencies]\nengine = {{ package = \"gpui-rsx-gpui-stub\", path = {:?} }}\n\
                 gpui-rsx = {{ path = {root:?} }}\n[workspace]\n",
                root.join("crates/gpui-stub"),
            ),
        )
        .unwrap();
        project
    }

    fn check(&self, case: &str, expected: bool) {
        let (success, errors) = self.run();
        assert_eq!(success, expected, "{case}: {errors}");
        if !expected {
            assert!(errors.contains("missing_engine"), "{case}: {errors}");
        }
    }

    fn run(&self) -> (bool, String) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target"))
            .join("configured-path-fixtures")
            .join(self.tag);
        let output = Command::new(env!("CARGO"))
            .args(["check", "--quiet", "--offline", "--manifest-path"])
            .arg(self.root.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", target)
            .output()
            .expect("run configured path cargo check");
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn parallel_projects_keep_their_source_directories_independent() {
    let barrier = std::sync::Barrier::new(8);
    let projects = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|worker| {
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    (0..16)
                        .map(|iteration| {
                            let project = Project::new("parallel-allocation");
                            let source = format!("worker {worker}, iteration {iteration}");
                            fs::write(project.root.join("src/lib.rs"), &source).unwrap();
                            (project, source)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    let roots: std::collections::HashSet<_> =
        projects.iter().map(|(project, _)| &project.root).collect();
    assert_eq!(roots.len(), projects.len());
    for (project, source) in &projects {
        assert_eq!(
            fs::read_to_string(project.root.join("src/lib.rs")).unwrap(),
            *source
        );
    }
}

fn macro_cases() -> [(&'static str, &'static str); 5] {
    [
        (
            "markup",
            "use engine::{prelude::*, *};
             pub fn render() -> impl IntoElement {
                 gpui_rsx::rsx! { <div key={String::from(\"row\")} onClick={|_, _, _| {}} /> }
             }",
        ),
        (
            "component",
            "#[gpui_rsx::component]
             pub struct Card { #[children] children: Vec<engine::AnyElement> }
             impl engine::RenderOnce for Card {
                 fn render(self, _: &mut engine::Window, _: &mut engine::App) -> impl engine::IntoElement {
                     engine::div()
                 }
             }",
        ),
        (
            "listener",
            "pub fn listen(cx: &engine::Context<'_, usize>) {
                 let _ = gpui_rsx::rsx_listener!(cx, notify, |_, _: &(), _, _| {});
             }",
        ),
        (
            "update",
            "pub fn update(view: &mut usize, cx: &mut engine::Context<'_, usize>) {
                 gpui_rsx::rsx_update!(view, cx, |view, _| *view += 1);
             }",
        ),
        ("runtime", "gpui_rsx::rsx_runtime!();"),
    ]
}

#[test]
fn every_macro_tracks_path_config_changes_and_explicit_runtime_paths_override_them() {
    let project = Project::new("path-config-changes");
    for (case, source) in macro_cases() {
        fs::write(
            project.root.join("src/lib.rs"),
            format!("pub use engine as ui;\n{source}\n"),
        )
        .unwrap();
        for (path, expected) in [
            ("crate::ui", true),
            ("::missing_engine", false),
            ("crate::ui", true),
        ] {
            fs::write(
                project.root.join("rsx.toml"),
                format!("[options]\ngpui-path = \"{path}\"\n"),
            )
            .unwrap();
            project.check(case, expected);
        }
    }
    fs::write(
        project.root.join("src/lib.rs"),
        "gpui_rsx::rsx_runtime!(::engine);\n",
    )
    .unwrap();
    fs::write(
        project.root.join("rsx.toml"),
        "[options]\ngpui-path = \"::missing_engine\"\n",
    )
    .unwrap();
    project.check("explicit runtime override", true);
}

#[test]
fn build_script_tracks_config_creation_removal_and_recreation_for_every_macro() {
    let project = Project::new("build-script");
    fs::write(
        project.root.join("build.rs"),
        include_str!("../demo/build.rs"),
    )
    .unwrap();
    let config = project.root.join("rsx.toml");
    for (case, source) in macro_cases() {
        fs::write(
            project.root.join("src/lib.rs"),
            format!("extern crate engine as gpui;\npub use engine as ui;\n{source}\n"),
        )
        .unwrap();
        project.check(&format!("{case} without config"), true);
        fs::write(&config, "[options]\ngpui-path = \"::missing_engine\"\n").unwrap();
        project.check(&format!("{case} after config creation"), false);
        fs::write(&config, "[options]\ngpui-path = \"crate::ui\"\n").unwrap();
        project.check(&format!("{case} after config recovery"), true);
        fs::remove_file(&config).unwrap();
        project.check(&format!("{case} after config removal"), true);
        fs::write(&config, "[options]\ngpui-path = \"::missing_engine\"\n").unwrap();
        project.check(&format!("{case} after config recreation"), false);
        fs::remove_file(&config).unwrap();
    }
}

#[test]
fn conditional_and_dynamic_theme_classes_preserve_context_borrows() {
    let project = Project::new("theme-context-borrows");
    fs::write(
        project.root.join("rsx.toml"),
        "[colors]\nsurface = { expr = \"cx.theme().background\" }\n",
    )
    .unwrap();
    fs::write(
        project.root.join("src/lib.rs"),
        "extern crate engine as gpui;
         use engine::{prelude::*, *};
         struct Theme { background: u32 }
         struct Context;
         impl Context {
             fn theme(&self) -> Theme { Theme { background: 0 } }
         }
         pub fn render(cx: &mut Context, active: bool) -> impl IntoElement {
             let element = gpui_rsx::rsx! {
                 <div class={if active { \"bg-surface\" } else { \"bg-surface\" }} />
             };
             let dynamic = if active { \"flex\" } else { \"grid\" };
             let dynamic_element = gpui_rsx::rsx! {
                 <div class={format!(\"bg-surface {}\", dynamic)} />
             };
             let _ = cx.theme().background;
             let _ = dynamic_element;
             element
         }",
    )
    .unwrap();

    let (success, errors) = project.run();
    assert!(success, "theme expressions must not move `cx`: {errors}");
}

#[test]
fn component_keeps_its_item_when_the_config_is_invalid() {
    let project = Project::new("component-config-error");
    fs::write(
        project.root.join("src/lib.rs"),
        "#[gpui_rsx::component]
         pub struct Card { pub label: u8 }
         pub fn make() -> Card { Card { label: 1 } }",
    )
    .unwrap();
    fs::write(
        project.root.join("rsx.toml"),
        "[options]\ngpui-path = \"engine\"\n",
    )
    .unwrap();
    let (success, errors) = project.run();
    assert!(!success, "{errors}");
    assert!(errors.contains("absolute Rust path"), "{errors}");
    // The original item survives, so the configuration error is not buried.
    assert!(!errors.contains("cannot find"), "{errors}");
    assert!(
        !errors.contains("E0422") && !errors.contains("E0412"),
        "{errors}"
    );
}

#[test]
fn expression_macros_report_only_the_config_error_when_it_is_invalid() {
    let project = Project::new("expression-config-error");
    fs::write(
        project.root.join("src/lib.rs"),
        "pub fn update(view: &mut usize, cx: &mut engine::Context<'_, usize>) {
             gpui_rsx::rsx_update!(view, cx, |view, _| *view += 1);
         }",
    )
    .unwrap();
    fs::write(
        project.root.join("rsx.toml"),
        "[options]\ngpui-path = \"engine\"\n",
    )
    .unwrap();
    let (success, errors) = project.run();
    assert!(!success, "{errors}");
    assert!(errors.contains("absolute Rust path"), "{errors}");
}
