use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "rsx-config-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn write(&self, path: &str, source: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn run(&self, args: &[&str], input: &str) -> (Option<i32>, String, String) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cargo-rsx"))
            .args(args)
            .current_dir(&self.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        (
            output.status.code(),
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn unresolved_dependencies_block_expansion_and_migration_without_writing() {
    let project = Project::new();
    project.write(
        "Cargo.toml",
        "[package]\nname = \"unresolved\"\nversion = \"0.1.0\"\n\
         [dependencies]\nmissing-dependency = { path = \"missing-dependency\" }\n",
    );
    let source = "fn view(){rsx!{<div class=\"gap-4\"/>};}\n";
    project.write("src/lib.rs", source);
    let (code, stdout, stderr) = project.run(
        &["expand", "-", "--manifest-path", "Cargo.toml", "--offline"],
        "<div class=\"gap-4\"/>",
    );
    assert_eq!(code, Some(2), "{stdout} {stderr}");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("cannot determine spacing scale"),
        "{stderr}"
    );
    assert!(stderr.contains("missing-dependency"), "{stderr}");

    let (code, _, stderr) = project.run(&["migrate", "src/lib.rs", "--offline", "--write"], "");
    assert_eq!(code, Some(2), "{stderr}");
    assert_eq!(
        fs::read_to_string(project.0.join("src/lib.rs")).unwrap(),
        source
    );

    // An explicit scale does not depend on a resolvable Cargo graph.
    project.write("rsx.toml", "[options]\nspacing-scale = \"px\"\n");
    let (code, _, stderr) = project.run(&["migrate", "src/lib.rs", "--offline", "--write"], "");
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        fs::read_to_string(project.0.join("src/lib.rs")).unwrap(),
        source.replace("gap-4", "gap-1")
    );
}

#[test]
fn sibling_features_unify_for_expansion_and_prevent_a_second_migration() {
    let project = Project::new();
    project.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"plain\", \"tailwind\"]\nresolver = \"2\"\n",
    );
    let rsx = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let source = "fn view(){rsx!{<div class=\"gap-4\"/>};}\n";
    for (name, features) in [
        ("plain", ""),
        ("tailwind", ", features = [\"tailwind-scale\"]"),
    ] {
        project.write(
            &format!("{name}/Cargo.toml"),
            &format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
                 [dependencies]\nrsx = {{ package = \"gpui-rsx\", path = {rsx:?}{features} }}\n"
            ),
        );
        project.write(&format!("{name}/src/lib.rs"), source);
        project.write(&format!("{name}/view.rsx"), "<div class=\"gap-4\"/>");
    }

    // Neither the plain member nor any ancestor manifest names tailwind-scale.
    for name in ["plain", "tailwind"] {
        let (code, expanded, stderr) =
            project.run(&["expand", &format!("{name}/view.rsx"), "--offline"], "");
        assert_eq!(code, Some(0), "{stderr}");
        assert!(expanded.contains("px(16f32)"), "{name}: {expanded}");

        let file = format!("{name}/src/lib.rs");
        let (code, _, stderr) = project.run(&["migrate", &file, "--write", "--offline"], "");
        assert_eq!(code, Some(2), "{name}: {stderr}");
        assert!(stderr.contains("already uses Tailwind scale"), "{stderr}");
        assert_eq!(fs::read_to_string(project.0.join(file)).unwrap(), source);
    }

    // Explicit config still takes priority over the workspace's unified feature.
    project.write("plain/rsx.toml", "[options]\nspacing-scale = \"px\"\n");
    let (code, expanded, stderr) = project.run(&["expand", "plain/view.rsx", "--offline"], "");
    assert_eq!(code, Some(0), "{stderr}");
    assert!(expanded.contains("px(4f32)"), "{expanded}");
    let (code, expanded, stderr) =
        project.run(&["expand", "plain/view.rsx", "--offline", "--tailwind"], "");
    assert_eq!(code, Some(0), "{stderr}");
    assert!(expanded.contains("px(16f32)"), "{expanded}");

    // Without the feature anywhere in the graph, plain px migration is allowed.
    fs::remove_file(project.0.join("plain/rsx.toml")).unwrap();
    let manifest = fs::read_to_string(project.0.join("tailwind/Cargo.toml")).unwrap();
    project.write(
        "tailwind/Cargo.toml",
        &manifest.replace(", features = [\"tailwind-scale\"]", ""),
    );
    let (code, expanded, stderr) = project.run(&["expand", "plain/view.rsx", "--offline"], "");
    assert_eq!(code, Some(0), "{stderr}");
    assert!(expanded.contains("px(4f32)"), "{expanded}");
    let (code, _, stderr) =
        project.run(&["migrate", "plain/src/lib.rs", "--write", "--offline"], "");
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        fs::read_to_string(project.0.join("plain/src/lib.rs")).unwrap(),
        source.replace("gap-4", "gap-1")
    );
}

#[test]
fn explicit_stdin_uses_the_selected_manifest_theme_and_scale() {
    let project = Project::new();
    project.write(
        "Cargo.toml",
        "[package]\nname = \"unrelated\"\nversion = \"0.1.0\"\n[workspace]\n",
    );
    project.write("src/lib.rs", "");
    project.write(
        "rsx.toml",
        "[options]\nspacing-scale = \"px\"\n[colors]\nbrand = \"#ff0000\"\n",
    );
    project.write(
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[workspace]\n",
    );
    project.write("app/src/lib.rs", "");
    project.write(
        "app/rsx.toml",
        "[options]\nspacing-scale = \"tailwind\"\n[colors]\nbrand = \"#123456\"\n",
    );
    let source = "<div class=\"gap-4 bg-brand\"/>";
    let (code, expected, stderr) = project.run(
        &[
            "expand",
            "--manifest-path",
            "app/Cargo.toml",
            "--offline",
            "--strict",
        ],
        source,
    );
    assert_eq!(code, Some(0), "{stderr}");
    assert!(expected.contains("px(16f32)"), "{expected}");
    assert!(!expected.contains("bg_brand"), "{expected}");

    for manifest in [
        PathBuf::from("app/Cargo.toml"),
        project.0.join("app/Cargo.toml"),
    ] {
        let (code, actual, stderr) = project.run(
            &[
                "expand",
                "-",
                "--manifest-path",
                manifest.to_str().unwrap(),
                "--offline",
                "--strict",
            ],
            source,
        );
        assert_eq!(code, Some(0), "{stderr}");
        assert_eq!(actual, expected);
    }

    // File input continues to resolve its own package's config ahead of the manifest.
    project.write("app/view.rsx", source);
    let (code, actual, stderr) = project.run(
        &[
            "expand",
            "app/view.rsx",
            "--manifest-path",
            "Cargo.toml",
            "--offline",
            "--strict",
        ],
        "",
    );
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(actual, expected);

    // Without --manifest-path both stdin forms use the current package's config.
    let (code, expected, stderr) = project.run(&["expand", "--offline", "--strict"], source);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(expected.contains("px(4f32)"), "{expected}");
    let (code, actual, stderr) = project.run(&["expand", "-", "--offline", "--strict"], source);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(actual, expected);
}
