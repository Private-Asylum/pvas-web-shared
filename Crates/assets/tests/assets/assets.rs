//! Keeps the embedded list honest against the vendored Yeti build.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use pvas_web_assets::{HOUSE, HOUSE_STYLESHEET, YETI};

/// Every module and theme a Yeti update brings must be embedded, or a page linking it would 404
/// with nothing failing at build time.
#[test]
fn every_module_and_theme_is_embedded() -> io::Result<()> {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/yeti");
    let embedded: BTreeSet<&str> = YETI.iter().map(|asset| asset.path).collect();

    let mut missing = Vec::new();
    for dir in ["js", "themes"] {
        for entry in fs::read_dir(vendor.join(dir))? {
            let name = entry?.file_name().to_string_lossy().into_owned();
            let path = format!("vendor/yeti/{dir}/{name}");
            if !embedded.contains(path.as_str()) {
                missing.push(path);
            }
        }
    }

    assert!(missing.is_empty(), "not embedded in YETI: {missing:?}");
    Ok(())
}

/// Whether `path` names a web font.
fn is_font(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("woff2"))
}

/// The record of which commit was built ships with it.
#[test]
fn vendored_commit_is_recorded() {
    let vendored = YETI
        .iter()
        .find(|asset| asset.path == "vendor/yeti/VENDORED")
        .map(|asset| String::from_utf8_lossy(asset.bytes).into_owned())
        .unwrap_or_default();
    assert!(
        vendored.contains("commit:"),
        "VENDORED does not record a commit"
    );
}

/// Every font the house stylesheet names ships beside it, and every vendored font is named, so a
/// font update cannot leave a face 404ing or a file shipped for nothing.
#[test]
fn house_fonts_match_the_stylesheet() -> io::Result<()> {
    let css = HOUSE
        .iter()
        .find(|asset| asset.path == HOUSE_STYLESHEET)
        .map(|asset| String::from_utf8_lossy(asset.bytes).into_owned())
        .unwrap_or_default();
    let named: BTreeSet<String> = css
        .split("url(\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .map(|url| format!("vendor/pvas/{url}"))
        .collect();
    let shipped: BTreeSet<String> = HOUSE
        .iter()
        .map(|asset| asset.path.to_owned())
        .filter(|path| is_font(path))
        .collect();
    assert!(!named.is_empty(), "the house stylesheet names no fonts");
    assert_eq!(named, shipped);

    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("vendor/fonts/geist-mono");
    let mut on_disk = BTreeSet::new();
    for entry in fs::read_dir(vendor)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if is_font(&name) {
            let _new = on_disk.insert(format!("vendor/pvas/fonts/{name}"));
        }
    }
    assert_eq!(
        on_disk, shipped,
        "a vendored font is not embedded, or one is missing"
    );
    Ok(())
}
