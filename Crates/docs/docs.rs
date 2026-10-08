//! Product documentation for Private Asylum sites.
//!
//! The source of every reference page is the manifest PVAS-DocGen writes from a plugin: one JSON
//! file per product, already gated (only narrative cleared with `// @doc` is in it) and already
//! sanitized. [`Manifest`] is its typed form; the contract is DocGen's `Docs/MANIFEST.md`.
//!
//! [`build_site`] turns a product's `docs.toml` (see [`config`]) into a whole site: its
//! hand-written pages, concept and reference pages from the manifest, a landing page and a
//! not-found page, with Pagefind search in every bar and every internal link checked. The `pvas-docs` binary runs it.

#[path = "config/config.rs"]
pub mod config;
#[path = "highlight/highlight.rs"]
pub mod highlight;
#[path = "layout/layout.rs"]
pub mod layout;
#[path = "links/links.rs"]
pub mod links;
#[path = "manifest/manifest.rs"]
mod manifest;
#[path = "markdown/markdown.rs"]
pub mod markdown;
#[path = "pages/pages.rs"]
mod pages;
#[path = "reference/reference.rs"]
pub mod reference;

pub use pages::build_site;

pub use manifest::{
    Concept, ConceptBlock, CppEnumValue, CppMember, CppParam, CppView, Direction, EnumValue,
    Function, Manifest, ManifestError, Module, Param, Product, Property, Returns, SCHEMA,
    TypeEntry, TypeKind,
};
