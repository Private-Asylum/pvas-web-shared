//! `pvas-docs`: builds and previews a product documentation site.
//!
//! Run from the product's documentation repository, which holds `docs.toml`:
//!
//!     pvas-docs                    # writes public/
//!     pvas-docs serve              # writes it and previews it at http://127.0.0.1:8080/
//!
//! The preview serves the site at the root; deployed, it lives under its base path (`/gantry/`),
//! so follow links from the landing page rather than typing paths.

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    pvas_web_site::run("pvas-docs", Path::new("public"), || {
        pvas_web_docs::build_site(Path::new("docs.toml"))
    })
}
