//! Keeps the embedded list honest against the vendored Yeti build.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use pvas_web_assets::YETI;

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
