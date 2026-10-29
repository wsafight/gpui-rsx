use super::{Options, input, mode, output};
use gpui_rsx_core::{ClassMode, SpacingScale, Theme};
use std::{
    borrow::Cow,
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};

fn content(opt: &Options, source: &str, mode: ClassMode) -> Result<String, String> {
    let calls = gpui_rsx_tools::macro_calls(source).unwrap_or_default();
    if calls.is_empty() {
        gpui_rsx_tools::preview::render(source, mode)
    } else {
        let call = calls.get(opt.index).ok_or_else(|| {
            format!(
                "macro index {} out of range ({} macros)",
                opt.index,
                calls.len()
            )
        })?;
        gpui_rsx_tools::preview::render(
            &source[call.start..call.end],
            gpui_rsx_tools::mode_for(&call.name, mode),
        )
    }
}

/// What a rendered preview depends on besides the source: a theme or scale change in
/// `rsx.toml` must refresh it too.
fn mode_key(mode: &Result<ClassMode, String>) -> Result<(SpacingScale, Option<Theme>), String> {
    mode.as_ref()
        .map(|mode| (mode.scale(), mode.theme().cloned()))
        .map_err(Clone::clone)
}

/// Source text plus the configuration that affects rendering, compared to decide whether
/// a watched request needs to re-render.
type WatchKey = (
    Result<String, String>,
    Result<(SpacingScale, Option<Theme>), String>,
);

fn watch_key(source: Result<String, String>, mode: Result<ClassMode, String>) -> WatchKey {
    (source, mode_key(&mode))
}

/// The refreshed `(key, body)` for an edit, or `None` when nothing changed.
///
/// Rendering errors become escaped HTML, so one broken edit keeps the preview server
/// serving instead of terminating it.
fn refreshed(
    opt: &Options,
    source: Result<String, String>,
    current_mode: Result<ClassMode, String>,
    cached: &WatchKey,
) -> Option<(WatchKey, String)> {
    let key = (source, mode_key(&current_mode));
    if &key == cached {
        return None;
    }
    let rendered = match (&key.0, current_mode) {
        (Ok(source), Ok(mode)) => content(opt, source, mode),
        (Err(error), _) => Err(error.clone()),
        (_, Err(error)) => Err(error),
    };
    Some((
        key,
        rendered.unwrap_or_else(|error| {
            format!("<pre>{}</pre>", gpui_rsx_tools::preview::escape(&error))
        }),
    ))
}

/// HTTP status line and body for a request line. Only the two preview routes are served.
fn response<'a>(route: &str, content: &'a str) -> (&'static str, Cow<'a, str>) {
    if route.starts_with("GET /content ") {
        ("200 OK", Cow::Borrowed(content))
    } else if route.starts_with("GET / ") {
        (
            "200 OK",
            Cow::Owned(gpui_rsx_tools::preview::page(content, true)),
        )
    } else {
        ("404 Not Found", Cow::Borrowed("Not found"))
    }
}

pub fn run(opt: &Options) -> Result<i32, String> {
    let path = match &opt.path {
        // A directory previews its first file with RSX macros, like the workspace default.
        Some(path) if !path.is_dir() => path.clone(),
        _ => super::files(opt)?
            .into_iter()
            .find(|path| {
                std::fs::read_to_string(path)
                    .ok()
                    .and_then(|s| gpui_rsx_tools::macro_calls(&s).ok())
                    .is_some_and(|v| !v.is_empty())
            })
            .ok_or("no RSX macros found; specify a file")?,
    };
    let source = input(Some(&path))?;
    let initial_mode = mode(opt, Some(&path));
    let initial = content(opt, &source, initial_mode.clone()?)?;
    if !opt.watch {
        output(opt, &gpui_rsx_tools::preview::page(&initial, false))?;
        return Ok(0);
    }
    let mut cached = (watch_key(Ok(source), initial_mode), initial);
    let listener = TcpListener::bind(("127.0.0.1", opt.port)).map_err(|e| e.to_string())?;
    eprintln!(
        "Static preview: http://{} ({}; Ctrl-C to stop)",
        listener.local_addr().map_err(|e| e.to_string())?,
        path.display()
    );
    for stream in listener.incoming() {
        // A failed or reset connection affects only that client; keep serving.
        let Ok(mut stream) = stream else {
            continue;
        };
        if stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .is_err()
        {
            continue;
        }
        let mut buffer = [0; 4096];
        let count = match stream.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => continue,
        };
        let request = String::from_utf8_lossy(&buffer[..count]);
        let route = request.lines().next().unwrap_or("");
        // Timestamps can stay unchanged across rapid or same-size edits. Compare
        // source text so those edits refresh without re-rendering unchanged input.
        let source = input(Some(&path));
        let current_mode = mode(opt, Some(&path));
        if let Some(updated) = refreshed(opt, source, current_mode, &cached.0) {
            cached = updated;
        }
        let (status, body) = response(route, &cached.1);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(index: usize) -> Options {
        Options {
            index,
            ..Options::default()
        }
    }

    #[test]
    fn bare_bodies_render_without_macro_selection() {
        let rendered =
            content(&options(0), r#"<div class="flex"/>"#, ClassMode::PERMISSIVE).unwrap();
        assert!(rendered.contains("data-rsx-tag=\"div\""), "{rendered}");
        assert!(rendered.contains("display:flex"), "{rendered}");
    }

    #[test]
    fn macro_indices_select_one_body_and_report_overflow() {
        let source = "fn view(){ rsx!{<div/>}; rsx!{<span/>}; }";
        let first = content(&options(0), source, ClassMode::PERMISSIVE).unwrap();
        assert!(first.contains("data-rsx-tag=\"div\""), "{first}");
        let second = content(&options(1), source, ClassMode::PERMISSIVE).unwrap();
        assert!(second.contains("data-rsx-tag=\"span\""), "{second}");
        let error = content(&options(2), source, ClassMode::PERMISSIVE).unwrap_err();
        assert!(error.contains("out of range (2 macros)"), "{error}");
    }

    #[test]
    fn macro_modes_apply_their_own_spacing_scale() {
        let source = "fn view(){ rsx_tw!{<div class=\"gap-4\"/>}; }";
        let scaled = content(&options(0), source, ClassMode::PERMISSIVE).unwrap();
        assert!(scaled.contains("gap:16px"), "{scaled}");

        let source = "fn view(){ rsx!{<div class=\"gap-4\"/>}; }";
        let plain = content(&options(0), source, ClassMode::PERMISSIVE).unwrap();
        assert!(plain.contains("gap:4px"), "{plain}");
    }

    #[test]
    fn mode_keys_track_the_scale_and_theme() {
        let mode = ClassMode::PERMISSIVE;
        let expected = mode.scale();
        let (scale, theme) = mode_key(&Ok(mode)).unwrap();
        assert_eq!(scale, expected);
        assert!(theme.is_none());

        let themed = ClassMode::PERMISSIVE.with_theme(Some(
            gpui_rsx_core::Theme::parse("[colors]\nbrand = \"#123456\"\n")
                .unwrap()
                .into(),
        ));
        let (_, theme) = mode_key(&Ok(themed)).unwrap();
        assert!(theme.is_some());

        assert_eq!(mode_key(&Err("bad theme".into())), Err("bad theme".into()));
    }

    #[test]
    fn responses_serve_preview_routes_and_404_others() {
        let (status, body) = response("GET /content HTTP/1.1", "<div/>");
        assert_eq!(status, "200 OK");
        assert!(matches!(body, Cow::Borrowed(_)));
        assert_eq!(body.as_ref(), "<div/>");

        let (status, body) = response("GET / HTTP/1.1", "<div/>");
        assert_eq!(status, "200 OK");
        assert!(matches!(body, Cow::Owned(_)));
        assert!(body.contains("<div/>"), "{body}");
        assert!(body.contains("setInterval"), "{body}");

        for route in [
            "GET /missing HTTP/1.1",
            "GET /contentx HTTP/1.1",
            "POST / HTTP/1.1",
            "GET /content",
            "",
        ] {
            let (status, body) = response(route, "<div/>");
            assert_eq!(status, "404 Not Found", "{route}");
            assert_eq!(body.as_ref(), "Not found", "{route}");
        }
    }

    #[test]
    fn refreshed_reuses_the_cached_body_when_the_key_matches() {
        let opt = options(0);
        let source = "fn view(){ rsx!{<div/>}; }".to_owned();
        let key = watch_key(Ok(source.clone()), Ok(ClassMode::PERMISSIVE));
        assert!(refreshed(&opt, Ok(source), Ok(ClassMode::PERMISSIVE), &key).is_none());

        let key = watch_key(Err("gone".into()), Err("bad theme".into()));
        assert!(refreshed(&opt, Err("gone".into()), Err("bad theme".into()), &key).is_none());
    }

    #[test]
    fn refreshed_rerenders_changed_sources_and_escapes_errors() {
        let opt = options(0);
        let cached = watch_key(
            Ok("fn view(){ rsx!{<div/>}; }".into()),
            Ok(ClassMode::PERMISSIVE),
        );

        let (key, body) = refreshed(
            &opt,
            Ok("fn view(){ rsx!{<span/>}; }".into()),
            Ok(ClassMode::PERMISSIVE),
            &cached,
        )
        .unwrap();
        assert!(body.contains("data-rsx-tag=\"span\""), "{body}");
        assert_ne!(key, cached);

        // A broken edit is shown escaped instead of taking down the preview server.
        let (_, body) = refreshed(
            &opt,
            Ok("fn view(){ rsx!{<div>".into()),
            Ok(ClassMode::PERMISSIVE),
            &cached,
        )
        .unwrap();
        assert!(
            body.starts_with("<pre>") && body.ends_with("</pre>"),
            "{body}"
        );
        // The error text is escaped, so the page cannot be broken by markup in it.
        let inner = body
            .strip_prefix("<pre>")
            .and_then(|body| body.strip_suffix("</pre>"))
            .unwrap();
        assert!(!inner.is_empty());
        assert!(!inner.contains('<'), "{inner}");

        // Configuration errors are rendered the same way.
        let (_, body) = refreshed(
            &opt,
            Ok(cached.0.clone().unwrap()),
            Err("bad theme".into()),
            &cached,
        )
        .unwrap();
        assert!(body.contains("bad theme"), "{body}");
    }

    #[test]
    fn refreshed_rerenders_when_the_theme_changes() {
        let opt = options(0);
        let source = "fn view(){ rsx!{<div class=\"bg-brand\"/>}; }";
        let plain = watch_key(Ok(source.into()), Ok(ClassMode::PERMISSIVE));
        let themed = ClassMode::PERMISSIVE.with_theme(Some(
            gpui_rsx_core::Theme::parse("[colors]\nbrand = \"#123456\"")
                .unwrap()
                .into(),
        ));

        let (key, body) = refreshed(&opt, Ok(source.into()), Ok(themed), &plain).unwrap();
        assert_ne!(key, plain);
        assert!(body.contains("#123456"), "{body}");
    }
}
