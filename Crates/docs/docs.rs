//! Product documentation for Private Asylum sites.
//!
//! The source of every reference page is the manifest PVAS-DocGen writes from a plugin: one JSON
//! file per product, already gated (only narrative cleared with `// @doc` is in it) and already
//! sanitized. [`Manifest`] is its typed form; the contract is DocGen's `Docs/MANIFEST.md`.

#[path = "manifest/manifest.rs"]
mod manifest;

pub use manifest::{
    Concept, ConceptBlock, CppEnumValue, CppMember, CppParam, CppView, Direction, EnumValue,
    Function, Manifest, ManifestError, Module, Param, Product, Property, Returns, SCHEMA,
    TypeEntry, TypeKind,
};
