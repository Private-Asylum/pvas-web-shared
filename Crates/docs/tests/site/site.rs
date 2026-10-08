//! A whole documentation site, built from a fixture project and Gantry's real manifest.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

use pvas_web_docs::build_site;
use pvas_web_site::Site;

const DOCS_TOML: &str = r#"
[site]
title = "Fixture"
slug = "fixture"
description = "A fixture site."
origin = "https://example.com"
repository = "https://github.com/example/fixture"
manifest = "manifest.json"
content = "content"

[home]
headline = "Headline."
lede = "Lede."
[[home.actions]]
label = "Start"
href = "/start/"
"#;

/// A fresh fixture project in a temporary folder, with `page` as its one content page.
fn project(test: &str, page: &str) -> io::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("pvas-web-docs-{test}-{}", process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(dir.join("content"))?;
    fs::write(dir.join("docs.toml"), DOCS_TOML)?;
    fs::write(
        dir.join("manifest.json"),
        include_str!("../fixtures/gantry.manifest.json"),
    )?;
    fs::write(dir.join("content/start.md"), page)?;
    Ok(dir)
}

/// The HTML of the file at `path` in `site`.
fn html<'s>(site: &'s Site, path: &str) -> Option<&'s str> {
    site.files()
        .iter()
        .find(|file| file.path == path)
        .and_then(|file| std::str::from_utf8(&file.bytes).ok())
}

#[test]
fn builds_every_kind_of_page() -> io::Result<()> {
    let dir = project(
        "builds",
        "+++\ntitle = \"Start\"\ngroup = \"Guide\"\n+++\n\nSee [the reference](/reference/).\n",
    )?;
    let site = build_site(&dir.join("docs.toml"))?;
    assert_eq!(site.base(), "/fixture/");
    assert!(site.search());

    for path in [
        "index.html",
        "404.html",
        "start/index.html",
        "concepts/index.html",
        "reference/index.html",
        "reference/ugantryexperiencedefinition/index.html",
        "css/docs.css",
        "vendor/yeti/yeti.min.css",
    ] {
        assert!(html(&site, path).is_some(), "missing {path}");
    }

    let definition =
        html(&site, "reference/ugantryexperiencedefinition/index.html").unwrap_or_default();
    assert!(definition.contains("GetDisplayName"));
    assert!(definition.contains("href=\"/fixture/reference/\""));
    assert!(definition.contains("<pagefind-modal-trigger>"));
    assert!(definition.contains("bundle-path=\"/fixture/pagefind/\""));

    let start = html(&site, "start/index.html").unwrap_or_default();
    assert!(start.contains("href=\"/fixture/reference/\""));
    assert!(start.contains("Edit this page on GitHub"));
    fs::remove_dir_all(&dir)
}

#[test]
fn a_broken_link_fails_the_build() -> io::Result<()> {
    let dir = project(
        "broken",
        "+++\ntitle = \"Start\"\n+++\n\n[gone](/reference/no-such-type/)\n",
    )?;
    let error = build_site(&dir.join("docs.toml"))
        .err()
        .map(|error| (error.kind(), error.to_string()));
    fs::remove_dir_all(&dir)?;

    let (kind, message) = error.unwrap_or((io::ErrorKind::Other, String::new()));
    assert_eq!(kind, io::ErrorKind::InvalidData);
    assert!(
        message.contains("/fixture/reference/no-such-type/"),
        "{message}"
    );
    Ok(())
}

#[test]
fn a_bad_config_names_the_file() -> io::Result<()> {
    let dir = project("config", "+++\ntitle = \"Start\"\n+++\n")?;
    fs::write(dir.join("docs.toml"), "[site]\ntitle = 1\n")?;
    let message = build_site(&dir.join("docs.toml"))
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();
    fs::remove_dir_all(&dir)?;
    assert!(message.contains("docs.toml"), "{message}");
    Ok(())
}
