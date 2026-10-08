//! Dioxus components shared by every Private Asylum site.
//!
//! Dioxus is used as a renderer only: components become HTML strings at build time and nothing
//! runs in the browser but the markup and Yeti's stylesheet. The components emit Yeti's own
//! markup (classes plus `data-*` attributes), so the layout, the sticky nav and its phone menu
//! all work without JavaScript.

#[path = "document/document.rs"]
mod document;
#[path = "footer/footer.rs"]
mod footer;
#[path = "nav/nav.rs"]
mod nav;

pub use document::{PageMeta, render_document};
pub use footer::{Footer, FooterProps};
pub use nav::{Nav, NavLink, NavProps};
