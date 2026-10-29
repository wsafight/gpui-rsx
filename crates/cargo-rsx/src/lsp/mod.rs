//! A synchronous stdio LSP server with incremental text sync and macro-body reuse.
mod document;
mod features;
mod markup;
mod protocol;
mod queue;

use document::{Document, diagnostics};
use protocol::{apply_content_changes, read, send};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{self, Write},
};

pub fn run() -> Result<(), String> {
    let (sender, receiver) = std::sync::mpsc::sync_channel(64);
    std::thread::spawn(move || {
        let mut input = io::BufReader::new(io::stdin().lock());
        loop {
            match read(&mut input) {
                Ok(Some(message)) => {
                    if sender.send(Ok(message)).is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
    });
    let mut out = io::stdout().lock();
    let mut docs: HashMap<String, Document> = HashMap::new();
    let mut initialized = false;
    let mut shutdown = false;
    let mut watch_files = false;
    let mut pending = None;
    while let Some(message) = pending.take().or_else(|| receiver.recv().ok()) {
        let mut message = message?;
        for _ in 0..64 {
            let Ok(next) = receiver.try_recv() else {
                break;
            };
            if let (Ok(left), Ok(Ok(right))) = (&mut message, &next)
                && queue::merge(left, right, &docs)
            {
                continue;
            }
            pending = Some(next);
            break;
        }
        let result = handle(
            message,
            &mut out,
            &mut docs,
            &mut initialized,
            &mut shutdown,
            &mut watch_files,
        );
        // Nothing kept across messages holds a span, so the per-thread source map that
        // `span-locations` parsing fills can be dropped; otherwise it grows on every edit.
        gpui_rsx_tools::release_span_memory();
        match result? {
            Flow::Continue => {}
            Flow::Exit(result) => return result,
        }
    }
    Ok(())
}

enum Flow {
    Continue,
    Exit(Result<(), String>),
}

fn handle(
    message: Result<Value, String>,
    out: &mut impl Write,
    docs: &mut HashMap<String, Document>,
    initialized: &mut bool,
    shutdown: &mut bool,
    watch_files: &mut bool,
) -> Result<Flow, String> {
    let message = match message {
        Ok(message) => message,
        Err(error) => {
            send(
                out,
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":error}}),
            )?;
            return Ok(Flow::Continue);
        }
    };
    let method = message["method"].as_str().unwrap_or("");
    let params = &message["params"];
    // Responses to our capability registration are not requests to answer.
    if method.is_empty() {
        return Ok(Flow::Continue);
    }
    if method == "initialized" && *watch_files {
        send(
            out,
            json!({"jsonrpc":"2.0","id":"gpui-rsx/watch-config","method":"client/registerCapability","params":{"registrations":[{"id":"gpui-rsx/config","method":"workspace/didChangeWatchedFiles","registerOptions":{"watchers":[{"globPattern":"**/rsx.toml","kind":7},{"globPattern":"**/Cargo.toml","kind":7}]}}]}}),
        )?;
        *watch_files = false;
        return Ok(Flow::Continue);
    }
    if method == "exit" {
        return Ok(Flow::Exit(if *shutdown {
            Ok(())
        } else {
            Err("exit before shutdown".into())
        }));
    }
    if method == "textDocument/didOpen" || method == "textDocument/didChange" {
        let doc = &params["textDocument"];
        let uri = doc["uri"].as_str().unwrap_or("").to_owned();
        let version = doc["version"].as_i64();
        if method.ends_with("didChange")
            && let Some(previous) = docs.get(&uri)
            && version
                .zip(previous.version)
                .is_some_and(|(new, old)| new <= old)
        {
            return Ok(Flow::Continue);
        }
        let text = if method.ends_with("didOpen") {
            doc["text"].as_str().map(str::to_owned)
        } else {
            match params["contentChanges"].as_array().map(|changes| {
                apply_content_changes(docs.get(&uri).map_or("", |doc| &doc.source), changes)
            }) {
                Some(Ok(text)) => Some(text),
                Some(Err(error)) => {
                    send(
                        out,
                        json!({"jsonrpc":"2.0","method":"window/logMessage","params":{"type":2,"message":format!("Ignored invalid edit: {error}")}}),
                    )?;
                    None
                }
                None => None,
            }
        };
        if let Some(text) = text {
            let mut document = match docs.get(&uri) {
                Some(previous) if method.ends_with("didChange") => previous.updated(text),
                _ => Document::new(&uri, &text),
            };
            document.version = version;
            send(
                out,
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":doc["version"],"diagnostics":diagnostics(&document)}}),
            )?;
            docs.insert(uri, document);
        }
    } else if method == "workspace/didChangeWatchedFiles" {
        // A manifest may have toggled `tailwind-scale`; resolve features again.
        crate::config::clear_feature_cache();
        let uris: Vec<_> = docs.keys().cloned().collect();
        for uri in uris {
            if let Some(document) = docs.get(&uri) {
                let mut refreshed = Document::new(&uri, &document.source);
                refreshed.version = document.version;
                let diagnostics = diagnostics(&refreshed);
                docs.insert(uri.clone(), refreshed);
                send(
                    out,
                    json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":diagnostics}}),
                )?;
            }
        }
    } else if method == "textDocument/didClose" {
        let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
        docs.remove(uri);
        send(
            out,
            json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}}),
        )?;
    } else if let Some(id) = message.get("id") {
        let response = if *shutdown {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32600,"message":"server is shutting down"}})
        } else if !*initialized && method != "initialize" {
            json!({"jsonrpc":"2.0","id":id,"error":{"code":-32002,"message":"server not initialized"}})
        } else {
            match method {
                "initialize" => {
                    *initialized = true;
                    *watch_files = params["capabilities"]["workspace"]["didChangeWatchedFiles"]["dynamicRegistration"]
                        == true;
                }
                "shutdown" => *shutdown = true,
                _ => {}
            }
            match features::request(method, params, docs) {
                Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
                Err(features::Unsupported) => {
                    json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("unsupported method {method}")}})
                }
            }
        };
        send(out, response)?;
    }
    Ok(Flow::Continue)
}
#[cfg(test)]
mod tests {
    use super::*;
    use protocol::range;

    #[test]
    fn incremental_notifications_keep_errors_and_ignore_stale_or_bad_updates() {
        let uri = "file:///tmp/incremental-rsx.rs";
        let source = "fn f(){rsx!{<div class=\"flex-colum\"/>};}";
        let start = source.find("flex-colum").unwrap();
        let mut server = Server::new();
        server.run(json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":source}}}));
        server.docs.get_mut(uri).unwrap().config_error = Some("bad theme".into());
        server.run(json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"range":range(source,start,start+10),"text":"flex-col"}]}}));
        assert!(server.docs[uri].source.contains("flex-col\""));
        assert!(server.docs[uri].analysis.diagnostics.is_empty());
        assert_eq!(server.docs[uri].config_error.as_deref(), Some("bad theme"));
        server.run(json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":1},"contentChanges":[{"text":"stale"}]}}));
        server.run(json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":3},"contentChanges":[{"range":{"start":{"line":9,"character":0},"end":{"line":9,"character":1}},"text":"bad"}]}}));
        assert!(server.docs[uri].source.contains("flex-col\""));
        assert_eq!(server.docs[uri].version, Some(2));
        assert!(server.output().contains("Ignored invalid edit"));
    }

    /// The mutable server state `handle` threads between messages.
    struct Server {
        out: Vec<u8>,
        docs: HashMap<String, Document>,
        initialized: bool,
        shutdown: bool,
        watch_files: bool,
    }

    impl Server {
        fn new() -> Self {
            Self {
                out: Vec::new(),
                docs: HashMap::new(),
                initialized: false,
                shutdown: false,
                watch_files: false,
            }
        }

        fn run(&mut self, message: Value) -> Flow {
            self.handle(Ok(message))
        }

        fn handle(&mut self, message: Result<Value, String>) -> Flow {
            handle(
                message,
                &mut self.out,
                &mut self.docs,
                &mut self.initialized,
                &mut self.shutdown,
                &mut self.watch_files,
            )
            .unwrap()
        }

        fn output(&self) -> String {
            String::from_utf8_lossy(&self.out).into_owned()
        }
    }

    #[test]
    fn registers_config_watchers_only_when_supported_and_ignores_the_response() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{"capabilities":{"workspace":{"didChangeWatchedFiles":{"dynamicRegistration":true}}}}}));
        server.run(json!({"method":"initialized","params":{}}));
        assert!(server.output().contains("client/registerCapability"));
        assert!(server.output().contains("**/rsx.toml"));
        assert!(server.output().contains("**/Cargo.toml"));
        let len = server.out.len();
        server.run(json!({"id":"gpui-rsx/watch-config","result":null}));
        server.run(json!({"method":"initialized","params":{}}));
        assert_eq!(server.out.len(), len);
    }

    #[test]
    fn handle_tracks_lifecycle_and_reports_framing_errors() {
        let mut server = Server::new();

        // A request before `initialize` is rejected with the spec's code.
        assert!(matches!(
            server.run(json!({"id":1,"method":"textDocument/hover","params":{}})),
            Flow::Continue
        ));
        assert!(server.output().contains("-32002"));

        server.run(json!({"id":2,"method":"initialize","params":{}}));
        assert!(server.initialized);

        // Opening a document publishes diagnostics and stores it.
        let uri = "file:///tmp/handle-test.rs";
        server.run(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":"fn f(){rsx!{<div class=\"flex-colum\"/>};}"}}}),
        );
        assert!(server.docs.contains_key(uri));
        assert!(server.output().contains("flex-colum"));

        // didChange keeps the resolved mode and uses the last content change.
        let before = server.docs[uri].mode.scale();
        server.run(
            json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"ignored"},{"text":"fn f(){rsx!{<div class=\"flex\"/>};}"}]}}),
        );
        assert!(server.docs[uri].source.contains("class=\"flex\""));
        assert_eq!(server.docs[uri].mode.scale(), before);

        // didClose clears diagnostics and forgets the document.
        server.run(json!({"method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}));
        assert!(!server.docs.contains_key(uri));

        // A malformed body is answered with -32700 instead of stopping the loop.
        server.handle(Err("bad json".into()));
        assert!(server.output().contains("-32700"));

        // `shutdown` is answered, then any further request is refused.
        server.run(json!({"id":3,"method":"shutdown","params":{}}));
        assert!(server.shutdown);
        server.run(json!({"id":4,"method":"textDocument/hover","params":{}}));
        assert!(server.output().contains("-32600"));

        // `exit` after `shutdown` succeeds; before it, it is an error.
        assert!(matches!(
            server.run(json!({"method":"exit"})),
            Flow::Exit(Ok(()))
        ));
        let mut fresh = Server::new();
        assert!(matches!(
            fresh.run(json!({"method":"exit"})),
            Flow::Exit(Err(_))
        ));
    }

    #[test]
    fn handle_refreshes_watched_files_and_ignores_unknown_notifications() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{}}));
        let uri = "file:///tmp/watched.rs";
        server.run(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":"fn f(){rsx!{<div class=\"flex-colum\"/>};}"}}}),
        );
        let before = server.output().len();

        // Re-resolving watched files republishes diagnostics for every open document.
        server.run(json!({"method":"workspace/didChangeWatchedFiles","params":{"changes":[]}}));
        let published = &server.output()[before..];
        assert!(
            published.contains("textDocument/publishDiagnostics"),
            "{published}"
        );
        assert!(published.contains("flex-colum"), "{published}");
        assert!(server.docs.contains_key(uri));

        // Unknown notifications (no id) are silently dropped.
        let before = server.output().len();
        server.run(json!({"method":"$/cancelRequest","params":{"id":1}}));
        assert_eq!(server.output().len(), before);
    }

    #[test]
    fn handle_maps_request_failures_to_jsonrpc_codes() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{}}));
        let uri = "file:///tmp/codes.rs";
        // Not a token stream, so formatting cannot run; that is a null result, not an error.
        server.run(
            json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":1,"text":"fn f( { rsx!{<div/>}; }"}}}),
        );
        server.run(json!({"id":2,"method":"textDocument/formatting","params":{"textDocument":{"uri":uri}}}));
        let response = server.output();
        assert!(response.contains(r#""result":null"#), "{response}");
        assert!(!response.contains("-32603"), "{response}");

        server.run(
            json!({"id":3,"method":"textDocument/rename","params":{"textDocument":{"uri":uri}}}),
        );
        assert!(server.output().contains("-32601"), "{}", server.output());
    }

    #[test]
    fn did_change_without_an_open_document_still_loads_it() {
        let mut server = Server::new();
        server.run(json!({"id":1,"method":"initialize","params":{}}));
        let uri = "file:///tmp/unopened.rs";

        // An editor may send changes for a file it never announced; the server must cope.
        server.run(
            json!({"method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":3},"contentChanges":[{"text":"fn f(){rsx!{<div class=\"flex-colum\"/>};}"}]}}),
        );
        assert!(server.docs.contains_key(uri));
        assert!(server.output().contains("flex-colum"));

        // didOpen without a `text` field is ignored rather than clearing the document.
        server.run(json!({"method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"version":4}}}));
        assert!(server.docs[uri].source.contains("flex-colum"));
    }
}
