//! A page's opening: the path it lives at, the headline ending in the house cursor, a lede, and
//! the actions.

use dioxus::prelude::*;

use crate::NavLink;

/// What the opening says.
#[derive(Props, Clone, Debug, PartialEq, Eq)]
pub struct IntroProps {
    /// A short path-like label above the headline, drawn as `~/<eyebrow>`: the product's slug, or
    /// the site's.
    eyebrow: String,
    /// The headline, the page's `h1`, with `id="headline"` so the section can be labelled by it.
    headline: String,
    /// One or two sentences under the headline.
    lede: String,
    /// Buttons under the lede: the first is the high-emphasis one, the rest medium.
    actions: Vec<NavLink>,
}

/// The opening of a landing page, as a labelled, centered section.
#[component]
pub fn Intro(props: IntroProps) -> Element {
    let IntroProps {
        eyebrow,
        headline,
        lede,
        actions,
    } = props;
    rsx! {
        section { class: "center", "aria-labelledby": "headline",
            div { class: "stack", "data-gap": "md",
                p { class: "pa-eyebrow", "{eyebrow}" }
                h1 { id: "headline", class: "pa-cursor", "{headline}" }
                p { class: "lede", "{lede}" }
                if !actions.is_empty() {
                    div { class: "cluster", "data-gap": "sm",
                        for (index, action) in actions.iter().enumerate() {
                            a {
                                class: "button",
                                href: "{action.href}",
                                "data-emphasis": if index == 0 { "high" } else { "medium" },
                                "{action.label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
