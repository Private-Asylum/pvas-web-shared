//! Everything Private Asylum's web projects share, behind one dependency.
//!
//! - [`components`]: Dioxus components every site is built from.
//! - [`assets`]: Yeti and the shared stylesheets, embedded.
//! - [`site`]: writing a site to disk, the `build`/`serve` command line, a local preview.
//! - [`docs`]: product documentation from PVAS-DocGen manifests.
//!
//! Each is also its own crate, for a project that wants only one of them.

pub use pvas_web_assets as assets;
pub use pvas_web_components as components;
pub use pvas_web_docs as docs;
pub use pvas_web_site as site;
