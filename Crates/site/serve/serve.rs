//! A local preview of the rendered site: `pvas-render serve`.
//!
//! Serves the output folder the way GitHub Pages does, closely enough to check a page: `/` and
//! folders resolve to their `index.html`, and anything missing gets `404.html` with a 404 status.
//! Bound to 127.0.0.1 only. Never part of what deploys.
//!
//! A site served under a base path elsewhere (`/gantry/`) is previewed at the root here.

#![expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "a preview server reports its address and its errors to the terminal"
)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tiny_http::{Header, Response, Server, StatusCode};

/// Serves `root` on `127.0.0.1:port` until the process is stopped.
///
/// # Errors
///
/// The port cannot be bound. Errors serving a single request are reported and skipped.
pub fn serve(root: &Path, port: u16) -> io::Result<()> {
    let server = Server::http(("127.0.0.1", port)).map_err(io::Error::other)?;
    println!(
        "Serving {} at http://127.0.0.1:{port}/ (Ctrl+C to stop)",
        root.display()
    );

    for request in server.incoming_requests() {
        let (status, file) = resolve(root, request.url()).map_or_else(
            || (StatusCode(404), root.join("404.html")),
            |file| (StatusCode(200), file),
        );

        let (status, file, body) = match fs::read(&file) {
            Ok(body) => (status, file, body),
            Err(error) => {
                eprintln!("error: reading {}: {error}", file.display());
                (
                    StatusCode(500),
                    PathBuf::from("error.txt"),
                    b"Internal error".to_vec(),
                )
            }
        };
        let mut response = Response::from_data(body).with_status_code(status);
        if let Some(header) = content_type(&file) {
            response = response.with_header(header);
        }

        if let Err(error) = request.respond(response) {
            eprintln!("error: responding: {error}");
        }
    }
    Ok(())
}

/// The file a URL names under `root`, or `None` when there is none.
///
/// Only plain path segments are accepted, so `..` cannot climb out of `root`. Query strings and
/// fragments are ignored; a folder resolves to its `index.html`.
fn resolve(root: &Path, url: &str) -> Option<PathBuf> {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    let mut file = root.to_path_buf();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        if segment == "." || segment == ".." || segment.contains('\\') {
            return None;
        }
        file.push(segment);
    }
    if file.is_dir() {
        file.push("index.html");
    }
    file.is_file().then_some(file)
}

/// The `Content-Type` header for a file, from its extension.
///
/// `None` only if `tiny_http` rejects the header, which fixed ASCII never is; a response then goes
/// out without one rather than the server panicking.
fn content_type(file: &Path) -> Option<Header> {
    let mime = match file.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        _ => "text/plain; charset=utf-8",
    };
    Header::from_bytes("Content-Type", mime).ok()
}
