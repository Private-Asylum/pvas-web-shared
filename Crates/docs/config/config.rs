//! A product documentation site's `docs.toml`.
//!
//! ```toml
//! [site]
//! title = "Gantry"
//! slug = "gantry"                                   # served at /gantry/; the repository's name
//! description = "One sentence for search results."
//! origin = "https://privateasylum.com"
//! repository = "https://github.com/Private-Asylum/gantry"
//! manifest = "data/manifest.json"                   # written by PVAS-DocGen
//! content = "content"                               # hand-written pages
//! styles = "styles"                                 # optional: the product's theme.css
//!
//! [[nav]]
//! label = "Private Asylum"
//! href = "https://privateasylum.com/"
//!
//! [sidebar]
//! groups = ["Getting Started", "Guides"]            # order of hand-written page groups
//!
//! [home]
//! headline = "Content that knows its order."
//! lede = "..."
//! [[home.actions]]
//! label = "Start here"
//! href = "/getting-started/"
//! [[home.cards]]
//! title = "Getting started"
//! text = "..."
//! href = "/getting-started/"
//! ```
//!
//! Site-absolute links (`/getting-started/`) are written without the base path; the renderer adds
//! it, so a product can move without its pages changing. Paths resolve against the directory the
//! config file is in.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// A whole `docs.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DocsConfig {
    /// Identity and inputs.
    pub site: SiteConfig,
    /// Extra links in the bar, after the site's own sections.
    #[serde(default)]
    pub nav: Vec<LinkConfig>,
    /// Sidebar ordering.
    #[serde(default)]
    pub sidebar: SidebarConfig,
    /// The landing page.
    pub home: HomeConfig,
}

/// Identity and inputs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SiteConfig {
    /// Product name, shown in the bar and every title.
    pub title: String,
    /// URL path segment, which is also the documentation repository's name.
    pub slug: String,
    /// One sentence for search results and link previews.
    pub description: String,
    /// The domain the site is served from, without a trailing slash.
    pub origin: String,
    /// The documentation repository, for edit links.
    pub repository: String,
    /// The branch edit links point at.
    #[serde(default = "default_branch")]
    pub branch: String,
    /// The PVAS-DocGen manifest, relative to the config file.
    pub manifest: PathBuf,
    /// The hand-written pages, relative to the config file.
    pub content: PathBuf,
    /// The product's own stylesheets, relative to the config file.
    #[serde(default)]
    pub styles: Option<PathBuf>,
}

/// The default branch edit links point at.
fn default_branch() -> String {
    "main".to_owned()
}

/// A link in the bar or on the landing page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LinkConfig {
    /// The link text.
    pub label: String,
    /// A site-absolute path (`/reference/`) or a full URL.
    pub href: String,
}

/// Sidebar ordering.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct SidebarConfig {
    /// The order hand-written page groups appear in; unlisted groups follow, alphabetically.
    #[serde(default)]
    pub groups: Vec<String>,
}

/// The landing page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HomeConfig {
    /// The one-line headline.
    pub headline: String,
    /// A sentence or two beneath it.
    pub lede: String,
    /// Buttons under the lede; the first is the primary one.
    #[serde(default)]
    pub actions: Vec<LinkConfig>,
    /// Cards pointing into the documentation.
    #[serde(default)]
    pub cards: Vec<CardConfig>,
}

/// One card on the landing page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CardConfig {
    /// The card's heading.
    pub title: String,
    /// A sentence or two.
    pub text: String,
    /// Where the card leads.
    pub href: String,
}

/// A loaded config and the directory its relative paths resolve against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    /// The config.
    pub config: DocsConfig,
    /// The directory the config file is in.
    pub root: PathBuf,
}

impl LoadedConfig {
    /// Reads and parses a `docs.toml`.
    ///
    /// # Errors
    ///
    /// The file cannot be read, or is not a valid config (`InvalidData`, with the parser's
    /// message).
    pub fn load(path: &Path) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        let config: DocsConfig = toml::from_str(&text).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", path.display()),
            )
        })?;
        let root = path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        Ok(Self { config, root })
    }

    /// The site's base path, e.g. `/gantry/`.
    #[must_use]
    pub fn base(&self) -> String {
        format!("/{}/", self.config.site.slug)
    }

    /// A path relative to the config file, resolved.
    #[must_use]
    pub fn resolve(&self, path: &Path) -> PathBuf {
        self.root.join(path)
    }

    /// A site-absolute path (`/reference/`) with the base path added; anything else unchanged.
    #[must_use]
    pub fn href(&self, href: &str) -> String {
        with_base(&self.base(), href)
    }
}

/// `href` with `base` prepended when it is site-absolute (`/x`, not `//x` or a full URL).
#[must_use]
pub fn with_base(base: &str, href: &str) -> String {
    if href.starts_with('/') && !href.starts_with("//") {
        format!("{}{href}", base.trim_end_matches('/'))
    } else {
        href.to_owned()
    }
}
