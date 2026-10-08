//! A card: a title, a sentence or two, and optionally where it leads and a label.

use dioxus::prelude::*;

/// What a card shows.
#[derive(Props, Clone, Debug, PartialEq, Eq)]
pub struct CardProps {
    /// The card's heading, an `h3`: cards sit under a section's `h2`.
    title: String,
    /// The body text.
    text: String,
    /// Where the card leads. A card with one is a link as a whole, its title ending in an arrow;
    /// without one it is a plain article.
    href: Option<String>,
    /// A short label beside the title, e.g. "coming soon".
    badge: Option<String>,
}

/// One card, for a Yeti `grid`.
#[component]
pub fn Card(props: CardProps) -> Element {
    let CardProps {
        title,
        text,
        href,
        badge,
    } = props;
    let heading = rsx! {
        if let Some(badge) = badge {
            div { class: "cluster", "data-justify": "between", "data-gap": "xs",
                h3 { "{title}" }
                span { class: "badge", "data-variant": "neutral", "{badge}" }
            }
        } else {
            h3 { "{title}" }
        }
    };
    rsx! {
        if let Some(href) = href {
            a { class: "card pa-card", href: "{href}",
                {heading}
                p { "{text}" }
            }
        } else {
            article { class: "card",
                {heading}
                p { "{text}" }
            }
        }
    }
}
