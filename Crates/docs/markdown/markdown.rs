//! Markdown to HTML: the hand-written pages and every prose field of the manifest.
//!
//! GitHub-flavoured (tables, strikethrough, task lists, footnotes, `> [!NOTE]` alerts), with
//! TOML front matter between `+++` lines and fenced code highlighted by syntect at build time.
//! Raw HTML in the source is escaped, never passed through. Headings get the ids comrak's own
//! anchorizer gives them, and the same ids are returned for the page's table of contents.

use comrak::nodes::NodeValue;
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Anchorizer, Arena, Options, format_html_with_plugins, parse_document};

use crate::config::with_base;
use crate::highlight::CLASS_PREFIX;

/// One heading of a rendered document, for its table of contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 1 to 6.
    pub level: u8,
    /// The `id` the heading carries in the HTML.
    pub id: String,
    /// Its plain text.
    pub text: String,
}

/// A rendered document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// The HTML.
    pub html: String,
    /// Every heading, in document order.
    pub headings: Vec<Heading>,
    /// The front matter between the `+++` lines, without them; empty when there is none.
    pub front_matter: String,
}

/// The front matter delimiter.
const FRONT_MATTER: &str = "+++";

/// Renders markdown. Built once per site: the highlighter loads its grammars on creation.
pub struct Markdown {
    /// Fenced-code highlighter, emitting classes styled by [`crate::highlight::stylesheet`].
    highlighter: SyntectAdapter,
}

impl std::fmt::Debug for Markdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Markdown").finish_non_exhaustive()
    }
}

impl Default for Markdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Markdown {
    /// A renderer with the default grammars.
    #[must_use]
    pub fn new() -> Self {
        Self {
            highlighter: SyntectAdapterBuilder::new()
                .css_with_class_prefix(CLASS_PREFIX)
                .build(),
        }
    }

    /// The parsing and rendering options every document shares.
    fn options() -> Options<'static> {
        let mut options = Options::default();
        options.extension.table = true;
        options.extension.strikethrough = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.footnotes = true;
        options.extension.alerts = true;
        options.extension.header_id_prefix = Some(String::new());
        options.extension.front_matter_delimiter = Some(FRONT_MATTER.to_owned());
        options.render.r#unsafe = false;
        options
    }

    /// Renders `markdown`, prefixing site-absolute links (`/reference/`) with `base`.
    #[must_use]
    pub fn render(&self, markdown: &str, base: &str) -> Rendered {
        let options = Self::options();
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &options);

        let mut anchorizer = Anchorizer::new();
        let mut headings = Vec::new();
        let mut front_matter = String::new();

        // Pre-order, which is document order: the ids match the ones comrak assigns, because
        // it anchorizes the same text in the same order while rendering.
        for node in root.descendants() {
            let mut data = node.data_mut();
            match &mut data.value {
                NodeValue::Heading(heading) => {
                    let level = heading.level;
                    drop(data);
                    let text = node.collect_text();
                    headings.push(Heading {
                        level,
                        id: anchorizer.anchorize(&text),
                        text,
                    });
                }
                NodeValue::Link(link) | NodeValue::Image(link) => {
                    link.url = with_base(base, &link.url);
                }
                NodeValue::FrontMatter(raw) => {
                    front_matter = strip_delimiters(raw);
                }
                _ => {}
            }
        }

        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&self.highlighter);
        let mut html = String::new();
        // Writing into a String cannot fail; an error here would be a comrak bug, and an empty
        // page is a better failure than a panic in the build.
        if format_html_with_plugins(root, &options, &mut html, &plugins).is_err() {
            html.clear();
        }

        Rendered {
            html,
            headings,
            front_matter,
        }
    }
}

/// Front matter as comrak keeps it, `+++` lines included, reduced to what is between them.
fn strip_delimiters(raw: &str) -> String {
    raw.lines()
        .filter(|line| line.trim() != FRONT_MATTER)
        .collect::<Vec<_>>()
        .join("\n")
}
