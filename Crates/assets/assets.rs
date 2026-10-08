//! Static files every Private Asylum site ships: the vendored Yeti build, and the house style.
//!
//! The files are embedded at compile time, so a site depends on this crate and gets them with no
//! vendor folder of its own and no copy step. Yeti itself is built from a pinned commit by
//! `scripts/update-yeti.sh` into `vendor/yeti/`, which records the commit in `VENDORED`. The house
//! style is `house/house.css` with its typeface, Geist Mono, vendored at a pinned version by
//! `scripts/update-fonts.sh` into `vendor/fonts/geist-mono/`.
//!
//! Paths here are relative to a site's root. A site served under a base path (`/gantry/`) joins
//! them onto its base; see [`YETI_STYLESHEET`].

/// One embedded file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asset {
    /// Where the file is written, relative to the site root, e.g. `vendor/yeti/yeti.min.css`.
    pub path: &'static str,
    /// The file's contents.
    pub bytes: &'static [u8],
}

/// Embeds files from `vendor/yeti/` under the same relative path in the output.
macro_rules! yeti {
    ($($file:literal),* $(,)?) => {
        &[$(
            Asset {
                path: concat!("vendor/yeti/", $file),
                bytes: include_bytes!(concat!("vendor/yeti/", $file)),
            }
        ),*]
    };
}

/// The Yeti files a page can link, plus its licence and the record of which commit was built.
/// Yeti's docs, editor data and starter files stay in the repository.
pub const YETI: &[Asset] = yeti![
    "yeti.css",
    "yeti.min.css",
    "yeti.js",
    "yeti.min.js",
    "yeti.min.js.map",
    "LICENSE",
    "VENDORED",
    "js/alert.js",
    "js/carousel.js",
    "js/demo.js",
    "js/dialog.js",
    "js/enter.js",
    "js/hover.js",
    "js/range.js",
    "js/tabs.js",
    "js/toc.js",
    "js/validate.js",
    "themes/sharp.css",
    "themes/soft.css",
];

/// Yeti's stylesheet, relative to the site root. Load it before any theme or site stylesheet.
pub const YETI_STYLESHEET: &str = "vendor/yeti/yeti.min.css";

/// The directory a Yeti module lives in, relative to the site root, e.g. for `toc.js`.
pub const YETI_MODULES: &str = "vendor/yeti/js";

/// Embeds one house-style file: `$from` in this crate, published at `vendor/pvas/$to`.
macro_rules! house {
    ($(($from:literal, $to:literal)),* $(,)?) => {
        &[$(
            Asset {
                path: concat!("vendor/pvas/", $to),
                bytes: include_bytes!($from),
            }
        ),*]
    };
}

/// The house style and its typeface, under `vendor/pvas/`. The stylesheet names its fonts by
/// relative paths, so the two must ship together.
pub const HOUSE: &[Asset] = house![
    ("house/house.css", "house.css"),
    (
        "vendor/fonts/geist-mono/geist-mono-latin-wght-normal.woff2",
        "fonts/geist-mono-latin-wght-normal.woff2"
    ),
    (
        "vendor/fonts/geist-mono/geist-mono-latin-wght-italic.woff2",
        "fonts/geist-mono-latin-wght-italic.woff2"
    ),
    (
        "vendor/fonts/geist-mono/geist-mono-latin-ext-wght-normal.woff2",
        "fonts/geist-mono-latin-ext-wght-normal.woff2"
    ),
    (
        "vendor/fonts/geist-mono/geist-mono-latin-ext-wght-italic.woff2",
        "fonts/geist-mono-latin-ext-wght-italic.woff2"
    ),
    (
        "vendor/fonts/geist-mono/geist-mono-symbols2-wght-normal.woff2",
        "fonts/geist-mono-symbols2-wght-normal.woff2"
    ),
    ("vendor/fonts/geist-mono/LICENSE", "fonts/LICENSE"),
    ("vendor/fonts/geist-mono/VENDORED", "fonts/VENDORED"),
];

/// The house stylesheet, relative to the site root. Load it after [`YETI_STYLESHEET`] and before
/// the site's own theme, which sets the hues.
pub const HOUSE_STYLESHEET: &str = "vendor/pvas/house.css";
