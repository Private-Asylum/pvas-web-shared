//! The site footer.

use dioxus::prelude::*;

use crate::NavLink;

/// What the footer shows.
#[derive(Props, Clone, Debug, PartialEq, Eq)]
pub struct FooterProps {
    /// The copyright line, e.g. "© 2026 Private Asylum LLC".
    copyright: String,
    /// Links on the far side of the footer.
    links: Vec<NavLink>,
}

/// The footer: the copyright line on one side, a few links on the other.
#[component]
pub fn Footer(props: FooterProps) -> Element {
    let FooterProps { copyright, links } = props;
    rsx! {
        footer { class: "box", "data-surface": "raised",
            div { class: "cluster", "data-justify": "between",
                small { "{copyright}" }
                div { class: "cluster", "data-gap": "md",
                    for link in links.iter() {
                        a { href: "{link.href}", "{link.label}" }
                    }
                }
            }
        }
    }
}
