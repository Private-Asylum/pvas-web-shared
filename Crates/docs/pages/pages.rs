//! A whole documentation site from a `docs.toml`: every page, the sidebar, the assets.
//!
//! Pages come from three places: the hand-written markdown under `content/`, the manifest's
//! concepts, and the manifest's types. The landing and not-found pages are built here; search is
//! Pagefind's modal, in every page's bar.
//! The site is written relative to its own root and served under its base path (`/gantry/`), so
//! output paths never carry the base and links always do.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use dioxus::prelude::*;
use serde::Deserialize;

use pvas_web_assets::{HOUSE_STYLESHEET, YETI_MODULES, YETI_STYLESHEET};
use pvas_web_components::NavLink;
use pvas_web_site::{Site, SiteFile};

use crate::Manifest;
use crate::config::LoadedConfig;
use crate::highlight;
use crate::layout::{Chrome, DocPage, SideGroup, SideLink, SideSection};
use crate::links;
use crate::markdown::Markdown;
use crate::reference::{Reference, concept_path, type_path};

/// The documentation stylesheet shared by every product: layout details Yeti leaves to the site.
const DOCS_CSS: &str = include_str!("../styles/docs.css");

/// Front matter of a hand-written page.
#[derive(Debug, Clone, Default, Deserialize)]
struct FrontMatter {
    /// The page title.
    #[serde(default)]
    title: String,
    /// The sidebar group the page belongs to.
    #[serde(default)]
    group: String,
    /// Position within the group; lower first, then by title.
    #[serde(default)]
    order: i64,
    /// One sentence for search results; the site's description when absent.
    #[serde(default)]
    description: String,
}

/// A hand-written page, rendered.
#[derive(Debug, Clone)]
struct ContentPage {
    /// Its front matter.
    front: FrontMatter,
    /// The rendered page, without the layout.
    page: DocPage,
}

/// Every `.md` file under `dir`, sorted, as paths relative to `dir`.
fn markdown_files(dir: &Path, relative: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir.join(relative))?.collect::<Result<_, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let path = relative.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            markdown_files(dir, &path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
    Ok(())
}

/// The site path a content file is served at: `guides/index.md` is `guides/`, `setup.md` is
/// `setup/`.
fn content_path(file: &Path) -> String {
    let stem = file.with_extension("");
    let mut parts: Vec<String> = stem
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.last().is_some_and(|last| last == "index") {
        let _index = parts.pop();
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}/", parts.join("/"))
    }
}

/// The date `file` last changed in git, `YYYY-MM-DD`, or `None` outside a repository.
fn last_updated(root: &Path, file: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%cs", "--"])
        .arg(file)
        .output()
        .ok()?;
    let date = String::from_utf8(output.stdout).ok()?.trim().to_owned();
    (output.status.success() && !date.is_empty()).then_some(date)
}

/// An `InvalidData` error with a message.
fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Builds the whole site described by the `docs.toml` at `config_path`, served under `/<slug>/`
/// and asking for a search index.
///
/// # Errors
///
/// Any input cannot be read or parsed, or a page links somewhere that does not exist.
pub fn build_site(config_path: &Path) -> io::Result<Site> {
    let loaded = LoadedConfig::load(config_path)?;
    let config = &loaded.config;
    let base = loaded.base();

    let manifest_path = loaded.resolve(&config.site.manifest);
    let manifest = Manifest::from_json(&fs::read_to_string(&manifest_path)?)
        .map_err(|error| invalid(format!("{}: {error}", manifest_path.display())))?;
    let markdown = Markdown::new();
    let reference = Reference::new(&manifest, &markdown, &base);

    let content = load_content(&loaded, &markdown)?;
    let sidebar = sidebar(&loaded, &content, &manifest, &reference);
    let chrome = chrome(&loaded, sidebar)?;

    let mut pages: Vec<DocPage> = content.iter().map(|page| page.page.clone()).collect();
    pages.extend(generated_pages(&loaded, &manifest, &reference));

    let mut rendered: Vec<(String, String)> = pages
        .iter()
        .map(|page| (format!("{}index.html", page.path), chrome.render(page)))
        .collect();
    rendered.push(("index.html".to_owned(), home(&loaded, &chrome)));
    rendered.push(("404.html".to_owned(), not_found(&loaded, &chrome)));

    let mut site = Site::new()
        .with_yeti()
        .with_house()
        .with_base(&base)
        .with_search();
    site.add(SiteFile::page(
        "css/docs.css",
        format!("{DOCS_CSS}\n{}", highlight::STYLESHEET),
    ));
    if let Some(styles) = &config.site.styles {
        site.extend(SiteFile::dir(&loaded.resolve(styles), "css")?);
    }
    for (path, html) in &rendered {
        site.add(SiteFile::page(path.clone(), html.clone()));
    }

    let files: BTreeSet<String> = site.files().iter().map(|file| file.path.clone()).collect();
    links::check(&rendered, &files, &base).map_err(|broken| {
        invalid(format!(
            "{} broken link(s):\n  {}",
            broken.len(),
            broken.join("\n  ")
        ))
    })?;
    Ok(site)
}

/// Every hand-written page, rendered without the layout.
fn load_content(loaded: &LoadedConfig, markdown: &Markdown) -> io::Result<Vec<ContentPage>> {
    let config = &loaded.config;
    let dir = loaded.resolve(&config.site.content);
    let mut files = Vec::new();
    markdown_files(&dir, Path::new(""), &mut files)?;

    let mut pages = Vec::new();
    for file in files {
        let path = content_path(&file);
        if path.is_empty() {
            // The landing page comes from `[home]` in the config, not from a file.
            continue;
        }
        let rendered = markdown.render(&fs::read_to_string(dir.join(&file))?, &loaded.base());
        let front: FrontMatter = toml::from_str(&rendered.front_matter)
            .map_err(|error| invalid(format!("{}: {error}", dir.join(&file).display())))?;
        let title = if front.title.is_empty() {
            rendered
                .headings
                .first()
                .map(|heading| heading.text.clone())
                .unwrap_or_default()
        } else {
            front.title.clone()
        };
        let source = config.site.content.join(&file);
        let source_str = source.to_string_lossy().replace('\\', "/");
        pages.push(ContentPage {
            page: DocPage {
                path,
                title,
                description: if front.description.is_empty() {
                    config.site.description.clone()
                } else {
                    front.description.clone()
                },
                html: rendered.html,
                headings: rendered.headings,
                crumbs: Vec::new(),
                edit_url: Some(format!(
                    "{}/edit/{}/{source_str}",
                    config.site.repository.trim_end_matches('/'),
                    config.site.branch
                )),
                updated: last_updated(&loaded.root, &source),
                provenance: None,
            },
            front,
        });
    }
    Ok(pages)
}

/// The sidebar: hand-written groups in configured order, then Concepts, then Reference.
fn sidebar(
    loaded: &LoadedConfig,
    content: &[ContentPage],
    manifest: &Manifest,
    reference: &Reference<'_>,
) -> Vec<SideGroup> {
    let order = &loaded.config.sidebar.groups;
    let mut names: Vec<&str> = content
        .iter()
        .map(|page| page.front.group.as_str())
        .collect();
    names.sort_by_key(|name| {
        (
            order.iter().position(|o| o == name).unwrap_or(usize::MAX),
            *name,
        )
    });
    names.dedup();

    let mut groups: Vec<SideGroup> = names
        .into_iter()
        .map(|name| {
            let mut pages: Vec<&ContentPage> = content
                .iter()
                .filter(|page| page.front.group == name)
                .collect();
            pages.sort_by(|a, b| {
                (a.front.order, &a.page.title).cmp(&(b.front.order, &b.page.title))
            });
            SideGroup {
                title: if name.is_empty() {
                    "Documentation".to_owned()
                } else {
                    name.to_owned()
                },
                sections: vec![SideSection {
                    title: None,
                    links: pages
                        .into_iter()
                        .map(|page| SideLink {
                            title: page.page.title.clone(),
                            path: page.page.path.clone(),
                        })
                        .collect(),
                }],
            }
        })
        .collect();

    let mut concept_links = vec![SideLink {
        title: "Overview".to_owned(),
        path: "concepts/".to_owned(),
    }];
    concept_links.extend(manifest.concepts.iter().map(|concept| SideLink {
        title: concept.title.clone(),
        path: concept_path(concept),
    }));
    groups.push(SideGroup {
        title: "Concepts".to_owned(),
        sections: vec![SideSection {
            title: None,
            links: concept_links,
        }],
    });

    let mut sections = vec![SideSection {
        title: None,
        links: vec![SideLink {
            title: "Index".to_owned(),
            path: "reference/".to_owned(),
        }],
    }];
    sections.extend(reference.by_module().into_iter().map(|(module, types)| {
        SideSection {
            title: Some(module),
            links: types
                .into_iter()
                .map(|entry| SideLink {
                    title: entry.id.clone(),
                    path: type_path(&entry.id),
                })
                .collect(),
        }
    }));
    groups.push(SideGroup {
        title: "Reference".to_owned(),
        sections,
    });
    groups
}

/// The bar, stylesheets and footer every page shares.
fn chrome(loaded: &LoadedConfig, sidebar: Vec<SideGroup>) -> io::Result<Chrome> {
    let config = &loaded.config;
    let base = loaded.base();

    // The bar: each hand-written group's first page, then Concepts, Reference, then extras.
    let mut nav: Vec<NavLink> = Vec::new();
    for group in &sidebar {
        if let Some(first) = group.links().next() {
            nav.push(NavLink::new(
                group.title.clone(),
                format!("{base}{}", first.path),
            ));
        }
    }
    nav.extend(
        config
            .nav
            .iter()
            .map(|link| NavLink::new(link.label.clone(), loaded.href(&link.href))),
    );

    let mut stylesheets = vec![
        format!("{base}{YETI_STYLESHEET}"),
        format!("{base}pagefind/pagefind-component-ui.css"),
        format!("{base}{HOUSE_STYLESHEET}"),
        format!("{base}css/docs.css"),
    ];
    if let Some(styles) = &config.site.styles {
        let mut names: Vec<String> = fs::read_dir(loaded.resolve(styles))?
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| {
                Path::new(name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("css"))
            })
            .collect();
        names.sort();
        stylesheets.extend(names.into_iter().map(|name| format!("{base}css/{name}")));
    }

    Ok(Chrome {
        title: config.site.title.clone(),
        base: base.clone(),
        nav,
        sidebar,
        stylesheets,
        toc_script: format!("{base}{YETI_MODULES}/toc.js"),
        search_bundle: format!("{base}pagefind/"),
        copyright: "© 2026 Private Asylum LLC".to_owned(),
        origin: config.site.origin.trim_end_matches('/').to_owned(),
    })
}

/// The concept and reference pages.
fn generated_pages(
    loaded: &LoadedConfig,
    manifest: &Manifest,
    reference: &Reference<'_>,
) -> Vec<DocPage> {
    let product = &manifest.product;
    let provenance = Some(format!(
        "Generated from {} {}{}",
        product.title,
        product.version,
        if product.engine.is_empty() {
            String::new()
        } else {
            format!(" on Unreal Engine {}", product.engine)
        }
    ));
    let description = loaded.config.site.description.clone();
    let concepts_crumb = vec![SideLink {
        title: "Concepts".to_owned(),
        path: "concepts/".to_owned(),
    }];
    let reference_crumb = vec![SideLink {
        title: "Reference".to_owned(),
        path: "reference/".to_owned(),
    }];

    let page = |path: String,
                title: String,
                description: String,
                article: crate::reference::Article,
                crumbs: Vec<SideLink>| DocPage {
        path,
        title,
        description,
        html: article.html,
        headings: article.headings,
        crumbs,
        edit_url: None,
        updated: None,
        provenance: provenance.clone(),
    };

    let mut pages = vec![
        page(
            "concepts/".to_owned(),
            "Concepts".to_owned(),
            description.clone(),
            reference.concepts_index_article(),
            Vec::new(),
        ),
        page(
            "reference/".to_owned(),
            "Reference".to_owned(),
            description.clone(),
            reference.index_article(),
            Vec::new(),
        ),
    ];
    pages.extend(manifest.concepts.iter().map(|concept| {
        page(
            concept_path(concept),
            concept.title.clone(),
            description.clone(),
            reference.concept_article(concept),
            concepts_crumb.clone(),
        )
    }));
    pages.extend(manifest.types.iter().map(|entry| {
        page(
            type_path(&entry.id),
            entry.id.clone(),
            if entry.summary.is_empty() {
                description.clone()
            } else {
                entry.summary.clone()
            },
            reference.type_article(entry),
            reference_crumb.clone(),
        )
    }));
    pages
}

/// A page outside the reading order, for the landing and not-found pages.
fn plain(title: &str, description: &str, path: &str) -> DocPage {
    DocPage {
        path: path.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        html: String::new(),
        headings: Vec::new(),
        crumbs: Vec::new(),
        edit_url: None,
        updated: None,
        provenance: None,
    }
}

/// The landing page: headline, actions, cards.
fn home(loaded: &LoadedConfig, chrome: &Chrome) -> String {
    let home = &loaded.config.home;
    let site = &loaded.config.site;
    let main = rsx! {
        main { id: "content", class: "stack", "data-gap": "3xl",
            section { class: "center", "aria-labelledby": "headline",
                div { class: "stack", "data-gap": "md",
                        // The eyebrow reads as the product's path (`~/gantry`); the headline ends
                        // in the house cursor.
                        p { class: "pa-docs-eyebrow", "{site.slug}" }
                        h1 { id: "headline", class: "pa-cursor", "{home.headline}" }
                        p { class: "lede", "{home.lede}" }
                        div { class: "cluster", "data-gap": "sm",
                            for (index, action) in home.actions.iter().enumerate() {
                                a {
                                    class: "button",
                                    href: loaded.href(&action.href),
                                    "data-emphasis": if index == 0 { "high" } else { "medium" },
                                    "{action.label}"
                                }
                            }
                        }
                }
            }
            if !home.cards.is_empty() {
                section { class: "center", "aria-label": "Sections",
                    div { class: "grid",
                        for card in home.cards.iter() {
                            a { class: "card pa-docs-card", href: loaded.href(&card.href),
                                h2 { "{card.title}" }
                                p { "{card.text}" }
                            }
                        }
                    }
                }
            }
        }
    };
    chrome.render_plain(&plain("", &site.description, ""), main)
}

/// The page GitHub Pages serves for a missing path under the base.
fn not_found(loaded: &LoadedConfig, chrome: &Chrome) -> String {
    let main = rsx! {
        main { id: "content", class: "center stack", "data-max": "md", "data-gap": "md",
            h1 { "Not found" }
            p { "There is nothing at this address in the {loaded.config.site.title} documentation." }
            div { class: "cluster", "data-gap": "sm",
                a { class: "button", href: loaded.href("/"), "Documentation home" }
                a { class: "button", "data-emphasis": "medium", href: loaded.href("/reference/"), "Reference" }
            }
        }
    };
    chrome.render_plain(
        &plain("Not found", "This page does not exist.", "404.html"),
        main,
    )
}
