//! `pvas-docs`: builds and previews a product documentation site.
//!
//! Run from the product's documentation repository, which holds `docs.toml`:
//!
//!     pvas-docs                    # writes public/ and indexes it for search
//!     pvas-docs serve              # the same, then previews it at http://127.0.0.1:8080/<slug>/
//!
//! The search index comes from Pagefind through `npx`, so Node must be installed.

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    pvas_web_site::run("pvas-docs", Path::new("public"), || {
        pvas_web_docs::build_site(Path::new("docs.toml"))
    })
}
