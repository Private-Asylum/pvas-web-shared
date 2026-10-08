//! The link check: every internal link, and every anchor it names, must exist.
//!
//! Runs over the rendered HTML before anything is written, so a broken link fails the build
//! instead of shipping. Internal links are site-absolute under the base (`/gantry/reference/`) or
//! same-page anchors (`#properties`). A relative link (`../reference/`) is reported too: written
//! from one page it breaks when the page moves, so the site uses absolute ones throughout.
//! External links and Pagefind's files, which are generated after the build, are not checked.

use std::collections::{BTreeMap, BTreeSet};

/// Values of `attribute="..."` in `html`, with `&amp;` decoded.
fn attribute_values<'h>(html: &'h str, attribute: &str) -> Vec<String> {
    let needle = format!(" {attribute}=\"");
    let mut values = Vec::new();
    let mut rest: &'h str = html;
    while let Some(start) = rest.find(&needle) {
        let after = rest.get(start + needle.len()..).unwrap_or_default();
        let Some(end) = after.find('"') else { break };
        values.push(after.get(..end).unwrap_or_default().replace("&amp;", "&"));
        rest = after.get(end..).unwrap_or_default();
    }
    values
}

/// The output file a site path names: `reference/` is `reference/index.html`.
fn file_for(path: &str) -> String {
    if path.is_empty() || path.ends_with('/') {
        format!("{path}index.html")
    } else {
        path.to_owned()
    }
}

/// Checks every page's links against `files` (every output path) and the pages' own ids.
///
/// `pages` are (output path, HTML) pairs. Returns one line per broken link.
///
/// # Errors
///
/// The broken links, as `page: link (reason)` lines.
pub fn check(
    pages: &[(String, String)],
    files: &BTreeSet<String>,
    base: &str,
) -> Result<(), Vec<String>> {
    let ids: BTreeMap<&str, BTreeSet<String>> = pages
        .iter()
        .map(|(path, html)| {
            (
                path.as_str(),
                attribute_values(html, "id").into_iter().collect(),
            )
        })
        .collect();

    let mut broken = Vec::new();
    for (page, html) in pages {
        let mut targets = attribute_values(html, "href");
        targets.extend(attribute_values(html, "src"));
        for target in targets {
            if let Some(reason) = problem(&target, page, files, &ids, base) {
                broken.push(format!("{page}: {target} ({reason})"));
            }
        }
    }
    broken.sort();
    broken.dedup();
    if broken.is_empty() {
        Ok(())
    } else {
        Err(broken)
    }
}

/// Why `target`, linked from `page`, is broken, or `None` when it is fine or not ours to check.
fn problem(
    target: &str,
    page: &str,
    files: &BTreeSet<String>,
    ids: &BTreeMap<&str, BTreeSet<String>>,
    base: &str,
) -> Option<&'static str> {
    let is_external = target.contains("://")
        || target.starts_with("mailto:")
        || target.starts_with("//")
        || target.starts_with("data:");
    if is_external {
        return None;
    }

    let (path, fragment) = target.split_once('#').unwrap_or((target, ""));
    let file = if path.is_empty() {
        page.to_owned()
    } else if let Some(within) = path.strip_prefix(base) {
        if within.starts_with("pagefind/") {
            return None;
        }
        file_for(within)
    } else if path.starts_with('/') {
        return Some("outside this site's base path");
    } else {
        return Some("relative link; use a site-absolute one");
    };

    if !files.contains(&file) {
        return Some("no such page or file");
    }
    if !fragment.is_empty()
        && ids
            .get(file.as_str())
            .is_some_and(|known| !known.contains(fragment))
    {
        return Some("no such anchor on that page");
    }
    None
}
