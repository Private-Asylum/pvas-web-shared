//! The HTML document every page is rendered into.

use dioxus::prelude::*;

/// What a page says about itself in its `<head>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageMeta {
    /// The `<title>`: the page's own name, then the site's.
    pub title: String,
    /// One sentence for search results and link previews.
    pub description: String,
    /// The page's absolute URL, for the canonical link. Empty for pages with no canonical
    /// address of their own, such as the not-found page.
    pub canonical: String,
    /// Stylesheets in load order, as site-absolute paths. Yeti first, then the theme, then the
    /// site's own rules, which sit outside Yeti's cascade layers and so win without a fight.
    pub stylesheets: Vec<String>,
}

/// Jumps and focused controls stop below the sticky nav: this is the nav's own height, so it
/// follows the type scale. Copied from Yeti's starter page, which derives it.
const SCROLL_PADDING: &str = "--yeti-scroll-padding: calc(max(var(--yeti-control-size), 1.5 * \
    var(--yeti-text-md) + 2 * var(--yeti-space-xs)) + 2 * var(--yeti-space-sm) + \
    var(--yeti-border-width) + var(--yeti-space-xs))";

/// Renders a complete HTML document around `body`, which should be a `body` element.
///
/// Dioxus renders elements, not documents, and has no `html` element at all: its apps mount
/// inside a page somebody else wrote. So the doctype and the `html` wrapper are written here,
/// and Dioxus renders what goes inside them.
pub fn render_document(meta: &PageMeta, body: Element) -> String {
    let meta = meta.clone();
    let head = dioxus_ssr::render_element(rsx! {
        meta { charset: "utf-8" }
        meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        title { "{meta.title}" }
        meta { name: "description", content: "{meta.description}" }
        if !meta.canonical.is_empty() {
            link { rel: "canonical", href: "{meta.canonical}" }
        }
        for sheet in meta.stylesheets.iter() {
            link { rel: "stylesheet", href: "{sheet}" }
        }
    });
    let body = dioxus_ssr::render_element(body);

    format!(
        "<!doctype html>\n<html lang=\"en\" style=\"{SCROLL_PADDING}\">\n<head>{head}</head>\n{body}\n</html>\n"
    )
}
