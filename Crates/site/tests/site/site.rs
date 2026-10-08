//! Writing a site: what lands on disk, what is replaced, and what is refused.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

use pvas_web_site::{Site, SiteFile, write};

/// A fresh, empty output folder for one test, unique to this process.
fn out_dir(test: &str) -> io::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pvas-web-site-{test}-{}", process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(dir)
}

#[test]
fn writes_pages_the_house_and_the_jekyll_marker() -> io::Result<()> {
    let out = out_dir("writes")?;
    let mut site = Site::new().with_house();
    site.add(SiteFile::page("index.html", "<!doctype html>".to_owned()));
    site.add(SiteFile::page(
        "docs/index.html",
        "<!doctype html>".to_owned(),
    ));

    let summary = write(&site, &out)?;

    assert_eq!(summary.pages, 2);
    assert!(out.join("docs/index.html").is_file());
    assert!(out.join("vendor/yeti/yeti.min.css").is_file());
    assert!(out.join("vendor/yeti/js/toc.js").is_file());
    assert!(out.join("vendor/pvas/house.css").is_file());
    assert!(
        out.join("vendor/pvas/fonts/geist-mono-latin-wght-normal.woff2")
            .is_file()
    );
    assert!(out.join(".nojekyll").is_file());
    fs::remove_dir_all(&out)
}

#[test]
fn replaces_the_folder_instead_of_merging() -> io::Result<()> {
    let out = out_dir("replaces")?;
    fs::create_dir_all(&out)?;
    fs::write(out.join("stale.html"), "a page the site no longer has")?;

    let mut site = Site::new();
    site.add(SiteFile::page("index.html", String::new()));
    let _summary = write(&site, &out)?;

    assert!(!out.join("stale.html").exists());
    fs::remove_dir_all(&out)
}

#[test]
fn refuses_paths_that_leave_the_folder() -> io::Result<()> {
    let out = out_dir("refuses")?;
    for path in [
        "../escape.html",
        "/etc/escape.html",
        "a/../../escape.html",
        "",
    ] {
        let mut site = Site::new();
        site.add(SiteFile::page(path, String::new()));
        let error = write(&site, &out).err().map(|error| error.kind());
        assert_eq!(
            error,
            Some(io::ErrorKind::InvalidInput),
            "accepted {path:?}"
        );
    }
    assert!(
        !out.exists(),
        "a refused site must not touch the output folder"
    );
    Ok(())
}

#[test]
fn normalizes_the_base_path() {
    assert_eq!(Site::new().base(), "/");
    assert_eq!(Site::default().base(), "/");
    assert_eq!(Site::new().with_base("gantry").base(), "/gantry/");
    assert_eq!(Site::new().with_base("/gantry/").base(), "/gantry/");
    assert_eq!(Site::new().with_base("/").base(), "/");
    assert!(!Site::new().search());
    assert!(Site::new().with_search().search());
}
