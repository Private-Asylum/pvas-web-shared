//! The stylesheet for highlighted code.
//!
//! Highlighting happens at build time: code arrives as spans carrying classes, and this
//! stylesheet colors them, a light theme by default and a dark one when the reader's system is
//! dark, matching Yeti, which follows the system too.

use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, css_for_theme_with_class_style};

/// The prefix on every highlighting class, so syntect's scope names (`source`, `string`) cannot
/// collide with a site's own classes.
pub const CLASS_PREFIX: &str = "hl-";

/// The theme for a light page.
const LIGHT_THEME: &str = "InspiredGitHub";

/// The theme for a dark page.
const DARK_THEME: &str = "base16-ocean.dark";

/// The highlighting stylesheet: the light theme, then the dark one inside a
/// `prefers-color-scheme: dark` query.
///
/// # Errors
///
/// A theme is missing from syntect's defaults, or syntect cannot render it to CSS.
pub fn stylesheet() -> Result<String, String> {
    let themes = ThemeSet::load_defaults();
    let style = ClassStyle::SpacedPrefixed {
        prefix: CLASS_PREFIX,
    };
    let css = |name: &str| {
        let theme = themes
            .themes
            .get(name)
            .ok_or_else(|| format!("syntect has no theme {name:?}"))?;
        css_for_theme_with_class_style(theme, style).map_err(|error| error.to_string())
    };
    Ok(format!(
        "{}\n@media (prefers-color-scheme: dark) {{\n{}\n}}\n",
        css(LIGHT_THEME)?,
        css(DARK_THEME)?
    ))
}
