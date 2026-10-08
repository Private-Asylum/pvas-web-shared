//! The site bar: Yeti's sticky `nav`, which folds its links behind a toggle on a narrow screen.

use dioxus::prelude::*;

/// One link in the bar or footer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavLink {
    /// The link text.
    pub label: String,
    /// Where it goes: a site-absolute path or a full URL.
    pub href: String,
}

impl NavLink {
    /// A link with this text to this address.
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
        }
    }
}

/// What the site bar shows.
#[derive(Props, Clone, Debug, PartialEq, Eq)]
pub struct NavProps {
    /// The site's name, linking home. Always visible, also on a phone.
    brand: NavLink,
    /// The links, inline on a wide screen and behind the toggle on a narrow one.
    links: Vec<NavLink>,
    /// An optional call to action, shown as a small button at the end of the bar.
    action: Option<NavLink>,
}

/// The site bar.
///
/// The menu is a native `popover` opened by the toggle, so it needs no script: Yeti shows the
/// links inline above its `md` threshold and behind the toggle below it.
#[component]
pub fn Nav(props: NavProps) -> Element {
    let NavProps {
        brand,
        links,
        action,
    } = props;
    rsx! {
        nav {
            class: "nav",
            "aria-label": "Site",
            "data-threshold": "md",
            "data-sticky": "",
            // A page-top bar sits on the edge; everything else that sticks keeps the offset.
            style: "--yeti-sticky-offset: 0",
            a { href: "{brand.href}", "data-brand": "", "{brand.label}" }
            button { r#type: "button", "popovertarget": "site-menu", "aria-label": "Menu",
                svg { "aria-hidden": "true", "viewBox": "0 0 16 16",
                    path {
                        d: "M2 4h12M2 8h12M2 12h12",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        "stroke-linecap": "round",
                    }
                }
            }
            ul { id: "site-menu", "popover": "", role: "list",
                li { "data-close": "",
                    button {
                        r#type: "button",
                        "popovertarget": "site-menu",
                        "popovertargetaction": "hide",
                        "aria-label": "Close",
                        "×"
                    }
                }
                for link in links.iter() {
                    li { a { href: "{link.href}", "{link.label}" } }
                }
            }
            if let Some(action) = action {
                div { "data-actions": "",
                    a { class: "button", href: "{action.href}", "data-size": "sm", "{action.label}" }
                }
            }
        }
    }
}
