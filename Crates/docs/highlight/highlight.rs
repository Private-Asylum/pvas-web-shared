//! The stylesheet for highlighted code.
//!
//! Highlighting happens at build time: syntect tags code with scope classes, and this stylesheet
//! colors them from the theme's hues, so code glows in the product's own phosphor and follows
//! any change to its theme.

/// The prefix on every highlighting class, so syntect's scope names (`source`, `string`) cannot
/// collide with a site's own classes.
pub const CLASS_PREFIX: &str = "hl-";

/// The highlighting stylesheet.
pub const STYLESHEET: &str = include_str!("highlight.css");
