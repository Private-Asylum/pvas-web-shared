//! Dioxus components shared by every Private Asylum site.
//!
//! Dioxus is used as a renderer only: components become HTML strings at build time and nothing
//! runs in the browser but the markup and Yeti's stylesheet. The components emit Yeti's own
//! markup (classes plus `data-*` attributes), so the layout, the sticky nav and its phone menu
//! all work without JavaScript. Their look is the house style in `pvas-web-assets`, which every
//! site loads; the few classes beyond Yeti's own (`pa-eyebrow`, `pa-cursor`, `pa-card`,
//! `pa-prose`) are defined there.

#[path = "card/card.rs"]
mod card;
#[path = "document/document.rs"]
mod document;
#[path = "footer/footer.rs"]
mod footer;
#[path = "intro/intro.rs"]
mod intro;
#[path = "nav/nav.rs"]
mod nav;

pub use card::{Card, CardProps};
pub use document::{PageMeta, render_document};
pub use footer::{Footer, FooterProps};
pub use intro::{Intro, IntroProps};
pub use nav::{Nav, NavLink, NavProps};
