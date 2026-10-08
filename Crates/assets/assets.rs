//! Static files every Private Asylum site ships: the vendored Yeti build.
//!
//! The files are embedded at compile time, so a site depends on this crate and gets Yeti with no
//! vendor folder of its own and no copy step. Yeti itself is built from a pinned commit by
//! `scripts/update-yeti.sh` into `vendor/yeti/`, which records the commit in `VENDORED`.
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
