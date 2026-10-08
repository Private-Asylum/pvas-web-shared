//! Reference and concept pages, rendered straight from the manifest.
//!
//! No markdown is generated in between: the manifest's structure becomes Yeti markup directly,
//! and only its prose fields (already markdown) go through the markdown renderer. Every heading's
//! id comes from one anchorizer per page, so the table of contents always matches.

use dioxus::prelude::*;

use comrak::Anchorizer;

use crate::config::with_base;
use crate::markdown::{Heading, Markdown};
use crate::{Concept, CppMember, Function, Manifest, Property, TypeEntry, TypeKind};

/// A rendered article: its HTML and its headings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Article {
    /// The article's HTML.
    pub html: String,
    /// Its headings, in order.
    pub headings: Vec<Heading>,
}

/// The URL-safe page name for a type or concept id.
#[must_use]
pub fn slug(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    let mut dash = false;
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').to_owned()
}

/// The path of a type's page within the site.
#[must_use]
pub fn type_path(id: &str) -> String {
    format!("reference/{}/", slug(id))
}

/// The path of a concept's page within the site.
#[must_use]
pub fn concept_path(concept: &Concept) -> String {
    format!("concepts/{}/", slug(&concept.title))
}

/// Human name for a type kind.
#[must_use]
pub const fn kind_label(kind: TypeKind) -> &'static str {
    match kind {
        TypeKind::Class => "Class",
        TypeKind::Struct => "Struct",
        TypeKind::Enum => "Enum",
        TypeKind::Interface => "Interface",
        TypeKind::Delegate => "Delegate",
        TypeKind::Namespace => "Namespace",
        TypeKind::Union => "Union",
    }
}

/// `Module/Public/Foo/Bar.h` as a user includes it: `Foo/Bar.h`.
fn include_path(header: &str, module: &str) -> String {
    let public = format!("{module}/Public/");
    header.strip_prefix(&public).map_or_else(
        || {
            header
                .split_once('/')
                .map_or(header, |(_, rest)| rest)
                .to_owned()
        },
        str::to_owned,
    )
}

/// Renders articles for one site.
pub struct Reference<'a> {
    /// The product's manifest.
    manifest: &'a Manifest,
    /// The markdown renderer, for prose fields.
    markdown: &'a Markdown,
    /// The site's base path.
    base: &'a str,
}

impl std::fmt::Debug for Reference<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reference")
            .field("base", &self.base)
            .finish_non_exhaustive()
    }
}

/// Collects one page's headings while it is built, with comrak's id scheme.
struct Toc {
    /// Assigns ids, unique within the page.
    anchorizer: Anchorizer,
    /// The headings so far.
    headings: Vec<Heading>,
}

impl Toc {
    /// An empty table of contents.
    fn new() -> Self {
        Self {
            anchorizer: Anchorizer::new(),
            headings: Vec::new(),
        }
    }

    /// Records a heading and returns the id to give it.
    fn add(&mut self, level: u8, text: &str) -> String {
        let id = self.anchorizer.anchorize(text);
        self.headings.push(Heading {
            level,
            id: id.clone(),
            text: text.to_owned(),
        });
        id
    }
}

impl<'a> Reference<'a> {
    /// A renderer for `manifest`'s pages under `base`.
    #[must_use]
    pub const fn new(manifest: &'a Manifest, markdown: &'a Markdown, base: &'a str) -> Self {
        Self {
            manifest,
            markdown,
            base,
        }
    }

    /// Markdown prose to HTML.
    fn prose(&self, text: &str) -> String {
        if text.is_empty() {
            String::new()
        } else {
            self.markdown.render(text, self.base).html
        }
    }

    /// C++ as a highlighted block.
    fn code(&self, cpp: &str) -> String {
        self.markdown
            .render(&format!("```cpp\n{cpp}\n```"), self.base)
            .html
    }

    /// A type name, linked when it names a type documented here.
    ///
    /// `const` and a trailing `*` or `&` are looked through (`const UFoo*` links to `UFoo`);
    /// anything more involved, such as a template argument, is shown but not linked.
    fn type_ref(&self, name: &str) -> Element {
        let core = name
            .trim()
            .trim_start_matches("const ")
            .trim_end_matches(['*', '&', ' ']);
        if self.manifest.type_by_id(core).is_some() {
            let href = with_base(self.base, &format!("/{}", type_path(core)));
            rsx! { a { href: "{href}", code { "{name}" } } }
        } else {
            rsx! { code { "{name}" } }
        }
    }

    /// Renders an element to a string.
    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    /// One type's article.
    #[must_use]
    pub fn type_article(&self, entry: &TypeEntry) -> Article {
        let mut toc = Toc::new();
        let title_id = toc.add(1, &entry.id);

        let mut badges = vec![kind_label(entry.kind).to_owned(), entry.module.clone()];
        badges.push(if entry.reflected {
            "Blueprint-visible".to_owned()
        } else {
            "C++ only".to_owned()
        });
        for flag in ["Abstract", "Deprecated", "Config"] {
            if entry.flags.iter().any(|f| f == flag) {
                badges.push(flag.to_lowercase());
            }
        }

        let include = (entry.kind != TypeKind::Namespace && !entry.header.is_empty()).then(|| {
            self.code(&format!(
                "#include \"{}\"",
                include_path(&entry.header, &entry.module)
            ))
        });

        let values_id = (!entry.values.is_empty()).then(|| toc.add(2, "Values"));
        let signature_id = entry.signature.as_ref().map(|_| toc.add(2, "Signature"));
        let properties_id = (!entry.properties.is_empty()).then(|| toc.add(2, "Properties"));
        let functions_id = (!entry.functions.is_empty()).then(|| toc.add(2, "Functions"));
        let function_ids: Vec<String> = entry
            .functions
            .iter()
            .map(|function| toc.add(3, &function.name))
            .collect();
        let members = entry
            .cpp
            .as_ref()
            .map(|cpp| cpp.members.as_slice())
            .unwrap_or_default();
        let cpp_id = (!members.is_empty())
            .then(|| toc.add(2, if entry.reflected { "C++" } else { "Members" }));
        let member_ids: Vec<String> = members
            .iter()
            .map(|member| toc.add(3, &member.name))
            .collect();

        let element = rsx! {
            h1 { id: "{title_id}", "{entry.id}" }
            div { class: "cluster", "data-gap": "xs",
                for badge in badges.iter() {
                    span { class: "badge", "data-variant": "neutral", "{badge}" }
                }
                if !entry.display_name.is_empty() {
                    span { class: "badge", "data-emphasis": "low", "shown as “{entry.display_name}”" }
                }
            }
            if let Some(include) = include {
                div { dangerous_inner_html: "{include}" }
            }
            div { dangerous_inner_html: "{self.prose(&entry.summary)}" }
            div { dangerous_inner_html: "{self.prose(&entry.description)}" }
            {self.relations(entry)}
            if let Some(id) = values_id {
                h2 { id: "{id}", "Values" }
                table { class: "table", "data-size": "sm",
                    thead { tr { th { "Value" } th { "Shown as" } th { "Description" } } }
                    tbody {
                        for value in entry.values.iter().filter(|value| !value.hidden) {
                            tr {
                                td { code { "{value.name}" } }
                                td { "{value.display_name}" }
                                td { dangerous_inner_html: "{self.prose(&value.doc)}" }
                            }
                        }
                    }
                }
            }
            if let (Some(id), Some(signature)) = (signature_id, entry.signature.as_ref()) {
                h2 { id: "{id}", "Signature" }
                {self.function_body(signature)}
            }
            if let Some(id) = properties_id {
                h2 { id: "{id}", "Properties" }
                {self.properties(&entry.properties)}
            }
            if let Some(id) = functions_id {
                h2 { id: "{id}", "Functions" }
                for (function, fid) in entry.functions.iter().zip(function_ids.iter()) {
                    h3 { id: "{fid}", code { "{function.name}" } }
                    {self.function_body(function)}
                }
            }
            if let Some(id) = cpp_id {
                h2 { id: "{id}", if entry.reflected { "C++" } else { "Members" } }
                if entry.reflected {
                    p { "Public C++ members that Blueprint does not see." }
                }
                {self.members(members, &member_ids)}
            }
        };

        Article {
            html: Self::html(element),
            headings: toc.headings,
        }
    }

    /// The C++ members reflection does not cover, each under its own heading.
    fn members(&self, members: &[CppMember], ids: &[String]) -> Element {
        rsx! {
            for (member, id) in members.iter().zip(ids.iter()) {
                h3 { id: "{id}", code { "{member.name}" } }
                if !member.signature.is_empty() {
                    div { dangerous_inner_html: "{self.code(&member.signature)}" }
                }
                div { dangerous_inner_html: "{self.prose(&member.summary)}" }
                div { dangerous_inner_html: "{self.prose(&member.description)}" }
                if !member.params.is_empty() {
                    ul {
                        for param in member.params.iter() {
                            li { code { "{param.name}" } ": " span { dangerous_inner_html: "{self.prose(&param.doc)}" } }
                        }
                    }
                }
                if !member.returns.is_empty() {
                    div { dangerous_inner_html: self.prose(&format!("**Returns:** {}", member.returns)) }
                }
            }
        }
    }

    /// Inherits, implements, declared in, category.
    fn relations(&self, entry: &TypeEntry) -> Element {
        let has_any = !entry.super_type.is_empty()
            || !entry.interfaces.is_empty()
            || !entry.owner.is_empty()
            || !entry.category.is_empty();
        if !has_any {
            return rsx! {};
        }
        rsx! {
            dl { class: "pa-docs-relations",
                if !entry.super_type.is_empty() {
                    dt { "Inherits" }
                    dd { {self.type_ref(&entry.super_type)} }
                }
                if !entry.interfaces.is_empty() {
                    dt { "Implements" }
                    dd {
                        for interface in entry.interfaces.iter() {
                            {self.type_ref(interface)}
                            " "
                        }
                    }
                }
                if !entry.owner.is_empty() {
                    dt { "Declared in" }
                    dd { {self.type_ref(&entry.owner)} }
                }
                if !entry.category.is_empty() {
                    dt { "Category" }
                    dd { "{entry.category}" }
                }
            }
        }
    }

    /// The properties table.
    fn properties(&self, properties: &[Property]) -> Element {
        rsx! {
            div { class: "scroller", tabindex: "0", "aria-label": "Properties",
                table { class: "table", "data-size": "sm",
                    thead { tr { th { "Property" } th { "Type" } th { "Default" } th { "Description" } } }
                    tbody {
                        for property in properties.iter() {
                            tr {
                                td {
                                    code { "{property.name}" }
                                    for tag in property_tags(property) {
                                        " "
                                        span { class: "badge", "data-size": "sm", "data-variant": "neutral", "{tag}" }
                                    }
                                }
                                td { {self.type_ref(&property.cpp_type)} }
                                td {
                                    if !property.default.is_empty() && property.default.len() <= 48 {
                                        code { "{property.default}" }
                                    }
                                }
                                td {
                                    div { dangerous_inner_html: "{self.prose(&property.summary)}" }
                                    div { dangerous_inner_html: "{self.prose(&property.description)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// A function's flags, prose and pins.
    fn function_body(&self, function: &Function) -> Element {
        let tags: Vec<&str> = [
            "BlueprintCallable",
            "BlueprintPure",
            "BlueprintImplementableEvent",
            "BlueprintNativeEvent",
            "Static",
            "Server",
            "Client",
            "NetMulticast",
        ]
        .into_iter()
        .filter(|tag| function.flags.iter().any(|flag| flag == tag))
        .collect();
        let has_pins = !function.params.is_empty() || function.returns.is_some();

        let has_badges = !tags.is_empty() || !function.category.is_empty();
        rsx! {
            if has_badges {
            div { class: "cluster", "data-gap": "xs",
                for tag in tags {
                    span { class: "badge", "data-size": "sm", "{tag}" }
                }
                if !function.category.is_empty() {
                    span { class: "badge", "data-size": "sm", "data-emphasis": "low", "{function.category}" }
                }
            }
            }
            div { dangerous_inner_html: "{self.prose(&function.summary)}" }
            div { dangerous_inner_html: "{self.prose(&function.description)}" }
            if has_pins {
                div { class: "scroller", tabindex: "0", "aria-label": "Pins of {function.name}",
                    table { class: "table", "data-size": "sm",
                        thead { tr { th { "Pin" } th { "Type" } th { "Direction" } th { "Description" } } }
                        tbody {
                            for pin in function.params.iter() {
                                tr {
                                    td {
                                        code { "{pin.name}" }
                                        if !pin.default.is_empty() {
                                            " = " code { "{pin.default}" }
                                        }
                                    }
                                    td { {self.type_ref(&pin.cpp_type)} }
                                    td { if matches!(pin.direction, crate::Direction::Out) { "out" } else { "in" } }
                                    td { dangerous_inner_html: "{self.prose(&pin.doc)}" }
                                }
                            }
                            if let Some(returns) = &function.returns {
                                tr {
                                    td { em { "Return Value" } }
                                    td { {self.type_ref(&returns.cpp_type)} }
                                    td { "out" }
                                    td { dangerous_inner_html: "{self.prose(&returns.doc)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The reference index: every type, by module.
    #[must_use]
    pub fn index_article(&self) -> Article {
        let mut toc = Toc::new();
        let title_id = toc.add(1, "Reference");
        let groups = self.by_module();
        let ids: Vec<String> = groups
            .iter()
            .map(|(module, _)| toc.add(2, module))
            .collect();
        let product = &self.manifest.product;

        let element = rsx! {
            h1 { id: "{title_id}", "Reference" }
            p {
                "Every documented type in {product.title} {product.version}, generated from the plugin's \
                 reflection data and headers. "
                strong { "Blueprint-visible" }
                " types are what you meet in the editor; "
                strong { "C++ only" }
                " types are available to code."
            }
            for ((module, types), id) in groups.iter().zip(ids.iter()) {
                h2 { id: "{id}", "{module}" }
                div { class: "scroller", tabindex: "0", "aria-label": "Types in {module}",
                    table { class: "table", "data-size": "sm",
                        thead { tr { th { "Type" } th { "Kind" } th { "Summary" } } }
                        tbody {
                            for entry in types.iter() {
                                tr {
                                    td { {self.type_ref(&entry.id)} }
                                    td {
                                        "{kind_label(entry.kind)}"
                                        if !entry.reflected { " · C++" }
                                    }
                                    td { dangerous_inner_html: "{self.prose(&entry.summary)}" }
                                }
                            }
                        }
                    }
                }
            }
        };
        Article {
            html: Self::html(element),
            headings: toc.headings,
        }
    }

    /// Types grouped by module, in the plugin's module order.
    #[must_use]
    pub fn by_module(&self) -> Vec<(String, Vec<&'a TypeEntry>)> {
        let order: Vec<&str> = self
            .manifest
            .product
            .modules
            .iter()
            .map(|module| module.name.as_str())
            .collect();
        let mut groups: Vec<(String, Vec<&TypeEntry>)> = Vec::new();
        for entry in &self.manifest.types {
            let module = if entry.module.is_empty() {
                "Other".to_owned()
            } else {
                entry.module.clone()
            };
            if let Some((_, types)) = groups.iter_mut().find(|(name, _)| *name == module) {
                types.push(entry);
            } else {
                groups.push((module, vec![entry]));
            }
        }
        groups.sort_by_key(|(name, _)| {
            (
                order.iter().position(|m| m == name).unwrap_or(usize::MAX),
                name.clone(),
            )
        });
        groups
    }

    /// One concept's article.
    #[must_use]
    pub fn concept_article(&self, concept: &Concept) -> Article {
        let mut toc = Toc::new();
        let title_id = toc.add(1, &concept.title);
        let many = concept.blocks.len() > 1;
        let block_ids: Vec<Option<String>> = concept
            .blocks
            .iter()
            .map(|block| {
                block
                    .symbol
                    .as_ref()
                    .filter(|_| many)
                    .map(|symbol| toc.add(2, symbol))
            })
            .collect();

        let element = rsx! {
            h1 { id: "{title_id}", "{concept.title}" }
            p { class: "pa-docs-source", code { "{concept.header}" } }
            for (block, id) in concept.blocks.iter().zip(block_ids.iter()) {
                if let (Some(id), Some(symbol)) = (id, block.symbol.as_ref()) {
                    h2 { id: "{id}", "{symbol}" }
                }
                div { dangerous_inner_html: "{self.prose(&block.markdown)}" }
                if let Some(symbol) = block.symbol.as_ref().filter(|s| self.manifest.type_by_id(s).is_some()) {
                    p { "See the reference: " {self.type_ref(symbol)} }
                }
            }
        };
        Article {
            html: Self::html(element),
            headings: toc.headings,
        }
    }

    /// The concepts index.
    #[must_use]
    pub fn concepts_index_article(&self) -> Article {
        let mut toc = Toc::new();
        let title_id = toc.add(1, "Concepts");
        let concepts = &self.manifest.concepts;
        let element = rsx! {
            h1 { id: "{title_id}", "Concepts" }
            p { "How the plugin's systems fit together, published from the design notes in its headers." }
            if concepts.is_empty() {
                div { class: "alert", "data-variant": "primary", role: "note",
                    p {
                        "Concept pages are published as each part of the design notes is reviewed. Until \
                         then, the "
                        a { href: with_base(self.base, "/reference/"), "reference" }
                        " covers every type."
                    }
                }
            } else {
                ul {
                    for concept in concepts.iter() {
                        li {
                            a { href: with_base(self.base, &format!("/{}", concept_path(concept))), "{concept.title}" }
                            " "
                            code { "{concept.header}" }
                        }
                    }
                }
            }
        };
        Article {
            html: Self::html(element),
            headings: toc.headings,
        }
    }
}

/// Short access notes for a property: read-only, read/write, config, replicated.
fn property_tags(property: &Property) -> Vec<&'static str> {
    let has = |flag: &str| property.flags.iter().any(|f| f == flag);
    let mut tags = Vec::new();
    if has("BlueprintVisible") {
        tags.push(if has("BlueprintReadOnly") {
            "read-only"
        } else {
            "read/write"
        });
    }
    if has("Config") {
        tags.push("config");
    }
    if has("Replicated") {
        tags.push("replicated");
    }
    tags
}
