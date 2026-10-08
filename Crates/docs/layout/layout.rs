//! The documentation page: bar, sidebar, article, table of contents, footer.
//!
//! Every part is Yeti markup. The sidebar groups are native `<details>` (Yeti's accordion), open
//! only for the group holding the current page, so a long reference stays short on a phone where
//! the sidebar stacks above the article. The table of contents is Yeti's `toc`, marked as the
//! reader scrolls by Yeti's optional `toc.js`; without it the links still work.

use dioxus::prelude::*;

use pvas_web_components::{Footer, Nav, NavLink, PageMeta, render_document};

use crate::markdown::Heading;

/// One link in the sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideLink {
    /// The link text.
    pub title: String,
    /// The page's path within the site, e.g. `reference/ugantryexperiencedefinition/`.
    pub path: String,
}

/// A run of sidebar links, optionally under a subheading (a reference module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideSection {
    /// The subheading; `None` for links that sit directly under the group.
    pub title: Option<String>,
    /// The links.
    pub links: Vec<SideLink>,
}

/// One sidebar group: Getting Started, Concepts, Reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideGroup {
    /// The group's heading.
    pub title: String,
    /// Its sections, in order.
    pub sections: Vec<SideSection>,
}

impl SideGroup {
    /// Every link in the group, in order.
    pub fn links(&self) -> impl Iterator<Item = &SideLink> {
        self.sections
            .iter()
            .flat_map(|section| section.links.iter())
    }

    /// True when `path` is one of the group's pages.
    fn contains(&self, path: &str) -> bool {
        self.links().any(|link| link.path == path)
    }
}

/// What the layout needs from the site as a whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chrome {
    /// Product name.
    pub title: String,
    /// Base path, e.g. `/gantry/`.
    pub base: String,
    /// The bar's links, base already applied.
    pub nav: Vec<NavLink>,
    /// The sidebar.
    pub sidebar: Vec<SideGroup>,
    /// Stylesheets, base already applied, in load order.
    pub stylesheets: Vec<String>,
    /// Yeti's table-of-contents module, base applied.
    pub toc_script: String,
    /// Where Pagefind's bundle is served, base applied, e.g. `/gantry/pagefind/`.
    pub search_bundle: String,
    /// The footer's copyright line.
    pub copyright: String,
    /// Canonical origin, without a trailing slash.
    pub origin: String,
}

/// One rendered documentation page, before the layout wraps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocPage {
    /// The page's path within the site: `""` for the landing page, `reference/x/` for a type.
    pub path: String,
    /// The page title, without the product name.
    pub title: String,
    /// One sentence for search results and previews.
    pub description: String,
    /// The article's HTML.
    pub html: String,
    /// The article's headings, for the table of contents.
    pub headings: Vec<Heading>,
    /// Trail from the landing page: (title, path) pairs, the page itself excluded.
    pub crumbs: Vec<SideLink>,
    /// Where to edit the page's source, for hand-written pages.
    pub edit_url: Option<String>,
    /// When the source last changed, `YYYY-MM-DD`, from git.
    pub updated: Option<String>,
    /// A line under the article, e.g. which plugin version the reference was taken from.
    pub provenance: Option<String>,
}

impl Chrome {
    /// A site path as a link: the base plus the path.
    fn href(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    /// The page before and after `path` in sidebar order.
    fn neighbours(&self, path: &str) -> (Option<&SideLink>, Option<&SideLink>) {
        let order: Vec<&SideLink> = self.sidebar.iter().flat_map(SideGroup::links).collect();
        let Some(index) = order.iter().position(|link| link.path == path) else {
            return (None, None);
        };
        let previous = index.checked_sub(1).and_then(|i| order.get(i)).copied();
        let next = order.get(index + 1).copied();
        (previous, next)
    }

    /// Head metadata for a page.
    fn meta(&self, page: &DocPage) -> PageMeta {
        PageMeta {
            title: if page.title.is_empty() {
                self.title.clone()
            } else {
                format!("{} · {}", page.title, self.title)
            },
            description: page.description.clone(),
            canonical: format!("{}{}{}", self.origin, self.base, page.path),
            stylesheets: self.stylesheets.clone(),
        }
    }

    /// The bar and footer around `main`.
    fn shell(&self, main: Element) -> Element {
        rsx! {
            body { class: "shell", "data-gap": "lg",
                a { href: "#content", "Skip to content" }
                Nav {
                    brand: NavLink::new(self.title.clone(), self.base.clone()),
                    links: self.nav.clone(),
                    action: None,
                    // Pagefind's search: a trigger in the bar (also Ctrl/Cmd+K) opening an
                    // accessible modal. The bundle is written by indexing the built site; without
                    // it these elements are undefined and render nothing.
                    extra_actions: rsx! { pagefind-modal-trigger {} },
                    threshold: Some("xl"),
                }
                // `preload` fetches the index with the page, so a query typed the moment the
                // modal opens is answered instead of being dropped while the index loads.
                pagefind-config { "bundle-path": "{self.search_bundle}", preload: "" }
                pagefind-modal {}
                {main}
                Footer { copyright: self.copyright.clone(), links: vec![] }
                script { r#type: "module", src: "{self.toc_script}" }
                script { r#type: "module", src: "{self.search_bundle}pagefind-component-ui.js" }
            }
        }
    }

    /// The sidebar, with `current` marked and its group open.
    fn sidebar(&self, current: &str) -> Element {
        rsx! {
            nav { class: "stack pa-docs-nav", "aria-label": "Documentation", "data-gap": "sm",
                for group in self.sidebar.iter() {
                    details { open: group.contains(current),
                        summary { "{group.title}" }
                        for section in group.sections.iter() {
                            if let Some(title) = &section.title {
                                p { class: "pa-docs-nav-section", "{title}" }
                            }
                            ul { role: "list",
                                for link in section.links.iter() {
                                    li {
                                        a {
                                            href: self.href(&link.path),
                                            "aria-current": if link.path == current { "page" } else { "false" },
                                            "{link.title}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The table of contents: the article's second- and third-level headings.
    fn toc(page: &DocPage) -> Element {
        let entries: Vec<&Heading> = page
            .headings
            .iter()
            .filter(|heading| matches!(heading.level, 2 | 3))
            .collect();
        if entries.is_empty() {
            return rsx! {};
        }
        rsx! {
            nav { class: "toc", "aria-label": "On this page", "data-size": "sm",
                p { class: "pa-docs-toc-title", "On this page" }
                ul { role: "list",
                    for heading in entries {
                        li { class: if heading.level == 3 { "pa-docs-toc-sub" } else { "" },
                            a { href: "#{heading.id}", "{heading.text}" }
                        }
                    }
                }
            }
        }
    }

    /// Renders a documentation page with the full layout.
    #[must_use]
    pub fn render(&self, page: &DocPage) -> String {
        let (previous, next) = self.neighbours(&page.path);
        let main = rsx! {
            // `center` gives the gutters on a narrow screen and caps the row on a wide one.
            div { class: "center", "data-max": "2xl", "data-gap": "md",
                div { class: "sidebar", "data-width": "xs", "data-gap": "lg",
                    div { class: "pa-docs-side", "data-sticky": "", {self.sidebar(&page.path)} }
                    div { class: "sidebar", "data-side": "end", "data-width": "xs", "data-gap": "lg",
                        main { id: "content", class: "stack pa-docs-article", "data-gap": "lg",
                            if !page.crumbs.is_empty() {
                                nav { class: "breadcrumbs", "aria-label": "Breadcrumb", "data-size": "sm",
                                    ol { role: "list",
                                        for crumb in page.crumbs.iter() {
                                            li { a { href: self.href(&crumb.path), "{crumb.title}" } }
                                        }
                                        li { "aria-current": "page", "{page.title}" }
                                    }
                                }
                            }
                            // Only the article body is indexed for search; Pagefind reads its attributes
                            // from this plain wrapper, so no Yeti element carries a foreign attribute.
                            div { class: "pa-docs-prose", "data-pagefind-body": "", dangerous_inner_html: "{page.html}" }
                            footer { class: "stack pa-docs-meta", "data-gap": "sm",
                                if previous.is_some() || next.is_some() {
                                div { class: "cluster", "data-justify": "between",
                                    if let Some(previous) = previous {
                                        a { class: "button", "data-emphasis": "low", rel: "prev", href: self.href(&previous.path),
                                            "← {previous.title}"
                                        }
                                    } else {
                                        span {}
                                    }
                                    if let Some(next) = next {
                                        a { class: "button", "data-emphasis": "low", rel: "next", href: self.href(&next.path),
                                            "{next.title} →"
                                        }
                                    }
                                }
                                }
                                if page.provenance.is_some() || page.updated.is_some() || page.edit_url.is_some() {
                                div { class: "cluster", "data-justify": "between",
                                    if let Some(provenance) = &page.provenance {
                                        small { "{provenance}" }
                                    }
                                    if let Some(updated) = &page.updated {
                                        small { "Last updated {updated}" }
                                    }
                                    if let Some(edit) = &page.edit_url {
                                        small { a { href: "{edit}", "Edit this page on GitHub" } }
                                    }
                                }
                                }
                            }
                        }
                        div { class: "pa-docs-toc-column", "data-sticky": "", {Self::toc(page)} }
                    }
                }
            }
        };
        render_document(&self.meta(page), self.shell(main))
    }

    /// Renders a page with the bar and footer but no sidebar: the landing, search and not-found
    /// pages, which are not part of the reading order.
    #[must_use]
    pub fn render_plain(&self, page: &DocPage, main: Element) -> String {
        render_document(&self.meta(page), self.shell(main))
    }
}
