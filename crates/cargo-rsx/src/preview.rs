use super::{Options, input, mode, output};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    time::Duration,
};

fn content(opt: &Options, path: &Path, source: &str) -> Result<String, String> {
    let mode = mode(opt, Some(path))?;
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

pub fn run(opt: &Options) -> Result<i32, String> {
    let path = match &opt.path {
        Some(path) => path.clone(),
        None => super::files(opt)?
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
    let initial = content(opt, &path, &source)?;
    if !opt.watch {
        output(opt, &gpui_rsx_tools::preview::page(&initial, false))?;
        return Ok(0);
    }
    let mut cached = (Ok(source), initial);
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
        if source != cached.0 {
            let rendered = match &source {
                Ok(source) => content(opt, &path, source),
                Err(error) => Err(error.clone()),
            };
            cached = (
                source,
                rendered.unwrap_or_else(|e| {
                    format!("<pre>{}</pre>", gpui_rsx_tools::preview::escape(&e))
                }),
            );
        }
        let fragment = cached.1.clone();
        let (status, body) = if route.starts_with("GET /content ") {
            ("200 OK", fragment)
        } else if route.starts_with("GET / ") {
            ("200 OK", gpui_rsx_tools::preview::page(&fragment, true))
        } else {
            ("404 Not Found", "Not found".into())
        };
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    }
    Ok(0)
}
