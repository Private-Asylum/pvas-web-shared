//! Turns rendered pages into a published static site.
//!
//! A site's own crate renders its pages (with `pvas-web-components`), collects them and its
//! static files into a [`Site`], and hands that to [`run`], which gives it the whole command
//! line: `build` writes the folder GitHub Pages publishes, `serve` writes it and previews it.

#[path = "cli/cli.rs"]
mod cli;
#[path = "output/output.rs"]
mod output;
#[path = "serve/serve.rs"]
mod serve;

pub use cli::run;
pub use output::{Site, SiteFile, Summary, write};
pub use serve::serve;
