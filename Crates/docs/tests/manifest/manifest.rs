//! The model reads a real manifest: Gantry's, as DocGen wrote it.

use pvas_web_docs::{Manifest, ManifestError, TypeKind};

/// Gantry's manifest, as published to its documentation site.
const GANTRY: &str = include_str!("../fixtures/gantry.manifest.json");

#[test]
fn reads_the_gantry_manifest() -> Result<(), ManifestError> {
    let manifest = Manifest::from_json(GANTRY)?;

    assert_eq!(manifest.product.slug, "gantry");
    assert_eq!(manifest.product.modules.len(), 6);
    assert!(manifest.types.iter().any(|entry| entry.reflected));
    assert!(manifest.types.iter().any(|entry| !entry.reflected));

    let definition = manifest.type_by_id("UGantryExperienceDefinition");
    assert_eq!(definition.map(|entry| entry.kind), Some(TypeKind::Class));
    assert_eq!(
        definition.map(|entry| entry.super_type.as_str()),
        Some("UPrimaryDataAsset")
    );

    let display_name = definition
        .and_then(|entry| entry.functions.iter().find(|f| f.name == "GetDisplayName"))
        .and_then(|function| function.returns.as_ref())
        .map(|returns| returns.cpp_type.as_str());
    assert_eq!(display_name, Some("FText"));
    Ok(())
}

#[test]
fn reads_enum_values_and_pin_directions() -> Result<(), ManifestError> {
    let manifest = Manifest::from_json(GANTRY)?;

    let compatibility = manifest.type_by_id("EGantryCompatibility");
    assert!(
        compatibility
            .is_some_and(|entry| entry.values.iter().any(|value| value.name == "Compatible"))
    );

    // Pins exist, and each one parsed: an unknown direction fails deserialization above.
    let pins = manifest
        .types
        .iter()
        .flat_map(|entry| entry.functions.iter())
        .flat_map(|function| function.params.iter())
        .count();
    assert!(pins > 0);
    Ok(())
}

#[test]
fn refuses_other_schemas() {
    let future = GANTRY.replacen("\"schema\": 1", "\"schema\": 2", 1);
    assert!(matches!(
        Manifest::from_json(&future),
        Err(ManifestError::Schema { found: 2 })
    ));
}
