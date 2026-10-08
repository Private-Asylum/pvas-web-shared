//! The search index: Pagefind, run over a written site.
//!
//! Pagefind is a Node tool, fetched and run through `npx` at a pinned version so a local build
//! and the deploy produce the same index. It indexes only what a page marks `data-pagefind-body`
//! and writes `pagefind/` into the folder it indexes.

use std::io;
use std::path::Path;
use std::process::Command;

/// The Pagefind release every site is indexed with.
pub const PAGEFIND_VERSION: &str = "1.5.2";

/// Builds the Pagefind index over the site written to `out`.
///
/// Quiet on success; on failure, the error carries Pagefind's own output.
///
/// # Errors
///
/// `npx` cannot be started (Node is not installed), or Pagefind fails.
pub fn index(out: &Path) -> io::Result<()> {
    let npx = if cfg!(windows) { "npx.cmd" } else { "npx" };
    let output = Command::new(npx)
        .args(["--yes", &format!("pagefind@{PAGEFIND_VERSION}"), "--site"])
        .arg(out)
        .output()
        .map_err(|error| io::Error::new(error.kind(), format!("running {npx}: {error}")))?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "pagefind failed ({}):\n{}{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}
