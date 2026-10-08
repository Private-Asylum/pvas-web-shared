//! The PVAS-DocGen manifest, schema 1.
//!
//! Mirrors DocGen's `Docs/MANIFEST.md` field for field. DocGen omits empty fields rather than
//! writing `""` or `[]`, so every optional field defaults. Unknown fields are ignored: the schema
//! number, not the field set, is what says whether this model can read a file.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

/// The manifest schema this model reads.
pub const SCHEMA: u32 = 1;

/// One product's documentation, as DocGen extracted it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Manifest {
    /// The schema the file was written in; must be [`SCHEMA`].
    pub schema: u32,
    /// The product the manifest describes.
    pub product: Product,
    /// One per header with narrative cleared for publication.
    #[serde(default)]
    pub concepts: Vec<Concept>,
    /// Every documented type, sorted by id.
    #[serde(default)]
    pub types: Vec<TypeEntry>,
    /// Sections written by a product's own DocGen extractors, keyed by section name.
    #[serde(default)]
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl Manifest {
    /// Parses a manifest and checks that it is a schema this model reads.
    ///
    /// # Errors
    ///
    /// The JSON does not parse into the model, or its schema is not [`SCHEMA`].
    pub fn from_json(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_str(json).map_err(ManifestError::Json)?;
        if manifest.schema == SCHEMA {
            Ok(manifest)
        } else {
            Err(ManifestError::Schema {
                found: manifest.schema,
            })
        }
    }

    /// The type with this id (its C++ name), if documented.
    #[must_use]
    pub fn type_by_id(&self, id: &str) -> Option<&TypeEntry> {
        self.types.iter().find(|entry| entry.id == id)
    }
}

/// Why a manifest could not be read.
#[derive(Debug)]
pub enum ManifestError {
    /// The file is not valid JSON for this model.
    Json(serde_json::Error),
    /// The file is valid but written in a schema this model does not read.
    Schema {
        /// The schema the file declares.
        found: u32,
    },
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "manifest is not valid: {error}"),
            Self::Schema { found } => write!(
                f,
                "manifest schema {found} is not supported (this renderer reads schema {SCHEMA})"
            ),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Schema { .. } => None,
        }
    }
}

/// The product a manifest describes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Product {
    /// URL path segment and documentation repository name, e.g. `gantry`.
    pub slug: String,
    /// Display name.
    pub title: String,
    /// The plugin's `VersionName`: the documentation's version axis.
    #[serde(default)]
    pub version: String,
    /// The `.uplugin`'s description.
    #[serde(default)]
    pub description: String,
    /// The engine the reflection was taken from, e.g. `5.6.1`.
    #[serde(default)]
    pub engine: String,
    /// Whether the plugin is marked beta.
    #[serde(default)]
    pub beta: bool,
    /// The plugin's modules, in `.uplugin` order.
    #[serde(default)]
    pub modules: Vec<Module>,
}

/// One module of the plugin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Module {
    /// Module name, e.g. `GantryCore`.
    pub name: String,
    /// Module type from the `.uplugin`, e.g. `Runtime` or `Editor`.
    #[serde(rename = "type")]
    pub kind: String,
}

/// One header's cleared narrative: a concept page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Concept {
    /// Header path relative to the plugin's `Source/`.
    pub header: String,
    /// The module the header belongs to.
    pub module: String,
    /// Page title: the header's file stem.
    pub title: String,
    /// The cleared blocks, in source order.
    pub blocks: Vec<ConceptBlock>,
}

/// One cleared narrative block.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ConceptBlock {
    /// The declaration the block documents; `None` for a free-standing block.
    #[serde(default)]
    pub symbol: Option<String>,
    /// The block as markdown.
    pub markdown: String,
}

/// What kind of type an entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeKind {
    /// A class.
    Class,
    /// A struct.
    Struct,
    /// An enum.
    Enum,
    /// An interface: the `I`-prefixed type C++ implements.
    Interface,
    /// A delegate signature.
    Delegate,
    /// A namespace with documented members.
    Namespace,
    /// A union.
    Union,
}

/// One documented type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeEntry {
    /// The C++ name, prefix included: the join key everywhere.
    pub id: String,
    /// What kind of type it is.
    pub kind: TypeKind,
    /// The module it is declared in.
    #[serde(default)]
    pub module: String,
    /// Header path relative to the plugin's `Source/`.
    #[serde(default)]
    pub header: String,
    /// Visible to Blueprint; false for the C++-only surface.
    #[serde(default)]
    pub reflected: bool,
    /// The narrative block is cleared, so `description` is present.
    #[serde(default)]
    pub published: bool,
    /// First sentence or `ShortTooltip`; present whenever the type is documented at all.
    #[serde(default)]
    pub summary: String,
    /// The cleared narrative; only when `published`.
    #[serde(default)]
    pub description: String,
    /// Display name in the editor, when it differs.
    #[serde(default)]
    pub display_name: String,
    /// Editor category.
    #[serde(default)]
    pub category: String,
    /// The parent type's id.
    #[serde(default, rename = "super")]
    pub super_type: String,
    /// For a delegate declared inside a class, that class's id.
    #[serde(default)]
    pub owner: String,
    /// Interfaces implemented, by id.
    #[serde(default)]
    pub interfaces: Vec<String>,
    /// Reflection flags, e.g. `Abstract`, `Config`.
    #[serde(default)]
    pub flags: Vec<String>,
    /// The config file a `Config` class reads.
    #[serde(default)]
    pub config_name: String,
    /// An enum's C++ form: `enumClass`, `namespaced` or `regular`.
    #[serde(default)]
    pub cpp_form: String,
    /// The remaining reflection metadata, passed through.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
    /// Properties, in declaration order.
    #[serde(default)]
    pub properties: Vec<Property>,
    /// Functions: the nodes a Blueprint user places.
    #[serde(default)]
    pub functions: Vec<Function>,
    /// An enum's values.
    #[serde(default)]
    pub values: Vec<EnumValue>,
    /// A delegate's signature.
    #[serde(default)]
    pub signature: Option<Function>,
    /// The Doxygen view: the C++ surface reflection does not cover.
    #[serde(default)]
    pub cpp: Option<CppView>,
}

/// One property.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    /// Property name.
    pub name: String,
    /// Display name in the editor, when it differs.
    #[serde(default)]
    pub display_name: String,
    /// C++ type, e.g. `TArray<FGameplayTag>`.
    #[serde(rename = "type")]
    pub cpp_type: String,
    /// Editor category.
    #[serde(default)]
    pub category: String,
    /// First sentence of the tooltip.
    #[serde(default)]
    pub summary: String,
    /// The rest of the tooltip.
    #[serde(default)]
    pub description: String,
    /// Exported default value from the CDO or a default-constructed struct.
    #[serde(default)]
    pub default: String,
    /// Property flags, e.g. `Edit`, `BlueprintReadOnly`, `Config`.
    #[serde(default)]
    pub flags: Vec<String>,
    /// The remaining metadata, e.g. `UIMin`, `Categories`.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
}

/// One function, as the node or event a Blueprint user sees.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    /// Function name.
    pub name: String,
    /// Display name in the editor, when it differs.
    #[serde(default)]
    pub display_name: String,
    /// Editor category.
    #[serde(default)]
    pub category: String,
    /// First sentence of the tooltip.
    #[serde(default)]
    pub summary: String,
    /// The rest of the tooltip.
    #[serde(default)]
    pub description: String,
    /// Function flags, e.g. `BlueprintCallable`, `BlueprintPure`, `Server`.
    #[serde(default)]
    pub flags: Vec<String>,
    /// Input and output pins, the return value excluded.
    #[serde(default)]
    pub params: Vec<Param>,
    /// The return pin, when there is one.
    #[serde(default)]
    pub returns: Option<Returns>,
    /// The remaining metadata, e.g. `CompactNodeTitle`, `WorldContext`.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
}

/// Which side of the node a pin is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// An input pin.
    In,
    /// An output pin, even though C++ takes it by reference.
    Out,
}

/// One pin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    /// Parameter name.
    pub name: String,
    /// Display name on the pin, when it differs.
    #[serde(default)]
    pub display_name: String,
    /// C++ type.
    #[serde(rename = "type")]
    pub cpp_type: String,
    /// Input or output.
    pub direction: Direction,
    /// Taken by reference in C++.
    #[serde(default)]
    pub by_ref: bool,
    /// Default value, when the pin has one.
    #[serde(default)]
    pub default: String,
    /// The pin's documentation.
    #[serde(default)]
    pub doc: String,
}

/// The return pin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Returns {
    /// C++ type.
    #[serde(default, rename = "type")]
    pub cpp_type: String,
    /// What it returns.
    #[serde(default)]
    pub doc: String,
}

/// One enum value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    /// The C++ name.
    pub name: String,
    /// The name the editor shows.
    #[serde(default)]
    pub display_name: String,
    /// The numeric value.
    #[serde(default)]
    pub value: Option<i64>,
    /// When to pick it.
    #[serde(default)]
    pub doc: String,
    /// Hidden from the editor.
    #[serde(default)]
    pub hidden: bool,
}

/// The Doxygen view of a type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CppView {
    /// Header path relative to the plugin's `Source/`.
    #[serde(default)]
    pub file: String,
    /// Line of the declaration.
    #[serde(default)]
    pub line: Option<u32>,
    /// Base classes.
    #[serde(default)]
    pub bases: Vec<String>,
    /// Public members reflection does not cover; never repeats a reflected member.
    #[serde(default)]
    pub members: Vec<CppMember>,
}

/// One public C++ member.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors the manifest, which records each C++ specifier as its own flag"
)]
pub struct CppMember {
    /// `function`, `variable`, `enum`, `typedef`, ...
    pub kind: String,
    /// Member name.
    pub name: String,
    /// The full declaration.
    #[serde(default)]
    pub signature: String,
    /// The brief description.
    #[serde(default)]
    pub summary: String,
    /// The detailed description.
    #[serde(default)]
    pub description: String,
    /// Declared `static`.
    #[serde(default, rename = "static")]
    pub is_static: bool,
    /// Declared `const`.
    #[serde(default, rename = "const")]
    pub is_const: bool,
    /// Declared `virtual`.
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    /// Declared `constexpr`.
    #[serde(default, rename = "constexpr")]
    pub is_constexpr: bool,
    /// Documented parameters.
    #[serde(default)]
    pub params: Vec<CppParam>,
    /// What it returns.
    #[serde(default)]
    pub returns: String,
    /// A nested enum's values.
    #[serde(default)]
    pub values: Vec<CppEnumValue>,
}

/// One documented C++ parameter.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CppParam {
    /// Parameter name.
    pub name: String,
    /// Its documentation.
    #[serde(default)]
    pub doc: String,
}

/// One value of an enum declared as a C++ member.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CppEnumValue {
    /// The value's name.
    pub name: String,
    /// Its initializer, e.g. `= 4`.
    #[serde(default)]
    pub initializer: String,
    /// Its documentation.
    #[serde(default)]
    pub doc: String,
}
