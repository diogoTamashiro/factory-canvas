use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, Catalog, CatalogId, CatalogMetadata, CategoryId,
    ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use factory_canvas::domain::document::{DocumentMetadata, DocumentMetadataError};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::persistence::blueprint_document::{
    decode_blueprint_document, encode_blueprint_document, BlueprintDocumentError,
    BlueprintNodeErrorKind,
};
use factory_canvas::persistence::factory_document::CatalogCompatibility;
use semver::Version;
use time::OffsetDateTime;

fn test_catalog() -> Catalog {
    let region_id = RegionId::new("test_region").unwrap();
    let base_id = BaseId::new("test_base").unwrap();
    let product_id = ProductId::new("test_product").unwrap();
    Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("test_catalog").unwrap(),
            Version::new(1, 2, 3),
            "Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(region_id.clone(), "Test Region")],
        vec![BaseDefinition::new(
            base_id,
            "Test Base",
            region_id,
            GridSize::new(30, 30).unwrap(),
        )],
        vec![BuildableDefinition::new(
            BuildableId::new("test_machine").unwrap(),
            "Test Machine",
            CategoryId::new("test_category").unwrap(),
            "TM",
            GridSize::new(2, 3).unwrap(),
            vec![product_id.clone()],
            None,
        )],
        vec![
            ProductDefinition::new(product_id, "Test Product"),
            ProductDefinition::new(ProductId::new("other_product").unwrap(), "Other Product"),
        ],
    )
    .unwrap()
}

use factory_canvas::domain::blueprint::{Blueprint, BlueprintId, Interface, Side};
use factory_canvas::domain::catalog::BuildableId;
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout};

fn test_blueprint_id() -> BlueprintId {
    BlueprintId::parse("blueprint_00000000000000000000000000000001").unwrap()
}

fn fixed_timestamp() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap()
}

fn test_metadata() -> DocumentMetadata {
    DocumentMetadata::new(
        "Test Blueprint",
        Some("A round-trip fixture"),
        fixed_timestamp(),
        fixed_timestamp(),
    )
    .unwrap()
}

fn blueprint_with_two_nodes(catalog: Catalog) -> Blueprint {
    let base_id = catalog.default_base_id().clone();
    let mut layout = FactoryLayout::new(catalog, base_id).unwrap();
    let first_id = EntityId::new(3);
    let second_id = EntityId::new(9);
    layout
        .place(BlockInstance::new(
            first_id,
            BuildableId::new("test_machine").unwrap(),
            GridPoint::new(4, 5),
            Rotation::Clockwise90,
        ))
        .unwrap();
    layout
        .set_production_target(first_id, Some(ProductId::new("test_product").unwrap()))
        .unwrap();
    layout
        .place(BlockInstance::new(
            second_id,
            BuildableId::new("test_machine").unwrap(),
            GridPoint::new(10, 5),
            Rotation::Zero,
        ))
        .unwrap();

    Blueprint::from_selection(
        &layout,
        vec![first_id, second_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .unwrap()
}

/// Same layout as `blueprint_with_two_nodes`, but captured with one
/// interface at (0, 0)/West — always a valid boundary point for any
/// `from_selection`-built blueprint, since node normalization guarantees
/// the bounding rectangle's own top-left corner is (0, 0).
fn blueprint_with_two_nodes_and_one_interface(catalog: Catalog) -> Blueprint {
    let base_id = catalog.default_base_id().clone();
    let mut layout = FactoryLayout::new(catalog, base_id).unwrap();
    let first_id = EntityId::new(3);
    let second_id = EntityId::new(9);
    layout
        .place(BlockInstance::new(
            first_id,
            BuildableId::new("test_machine").unwrap(),
            GridPoint::new(4, 5),
            Rotation::Clockwise90,
        ))
        .unwrap();
    layout
        .place(BlockInstance::new(
            second_id,
            BuildableId::new("test_machine").unwrap(),
            GridPoint::new(10, 5),
            Rotation::Zero,
        ))
        .unwrap();

    Blueprint::from_selection(
        &layout,
        vec![first_id, second_id],
        test_blueprint_id(),
        test_metadata(),
        vec![Interface::new("Input", GridPoint::new(0, 0), Side::West)],
    )
    .unwrap()
}

#[test]
fn blueprint_v1_round_trip_is_readable_and_preserves_independent_nodes() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());

    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let json = std::str::from_utf8(&encoded).unwrap();
    assert!(json.contains("\"schema_version\": 1"));
    assert!(json.contains("\"blueprint_id\": \"blueprint_"));
    assert!(json.contains("\"rotation_degrees\": 90"));
    assert!(json.ends_with('\n'));

    let loaded = decode_blueprint_document(&encoded, catalog).unwrap();

    assert_eq!(loaded.blueprint, blueprint);
    assert_eq!(loaded.compatibility, CatalogCompatibility::Exact);
}

#[test]
fn decoding_rejects_a_document_with_unknown_top_level_field() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["unknown_top_level_field"] = serde_json::json!("probe");
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert!(
        matches!(result, Err(BlueprintDocumentError::InvalidJson { .. })),
        "unknown field must be rejected, got {result:?}"
    );
}

#[test]
fn decoding_rejects_an_unsupported_schema_version() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["schema_version"] = serde_json::json!(2);
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert_eq!(
        result,
        Err(BlueprintDocumentError::UnsupportedSchemaVersion)
    );
}

#[test]
fn decoding_rejects_an_invalid_blueprint_id() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["blueprint_id"] = serde_json::json!("not-a-blueprint-id");
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert_eq!(result, Err(BlueprintDocumentError::InvalidBlueprintId));
}

#[test]
fn decoding_rejects_an_empty_nodes_array() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["nodes"] = serde_json::json!([]);
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert_eq!(result, Err(BlueprintDocumentError::EmptyNodes));
}

#[test]
fn decoding_rejects_non_canonical_local_node_ids() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    // A valid two-node blueprint has local IDs 1 and 2. Skip straight to 3,
    // which is non-canonical (not `1..=N`) even though it is still unique.
    value["nodes"][1]["id"] = serde_json::json!(3);
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert!(
        matches!(
            result,
            Err(BlueprintDocumentError::NonCanonicalNodeId { .. })
        ),
        "non-canonical local ID must be rejected, got {result:?}"
    );
}

#[test]
fn decoding_rejects_an_invalid_rotation_value() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["nodes"][0]["rotation_degrees"] = serde_json::json!(45);
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert!(
        matches!(result, Err(BlueprintDocumentError::InvalidRotation { .. })),
        "unsupported rotation degrees must be rejected, got {result:?}"
    );
}

#[test]
fn decoding_rejects_a_reference_to_an_unavailable_buildable() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["nodes"][0]["buildable_id"] = serde_json::json!("nonexistent_machine");
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert!(
        matches!(
            result,
            Err(BlueprintDocumentError::InvalidNode {
                kind: BlueprintNodeErrorKind::BuildableNotFound,
                ..
            })
        ),
        "reference to an unavailable buildable must be rejected, got {result:?}"
    );
}

#[test]
fn decoding_rejects_a_reference_to_an_unavailable_product() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["nodes"][0]["production_target"] = serde_json::json!("nonexistent_product");
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert!(
        matches!(
            result,
            Err(BlueprintDocumentError::InvalidNode {
                kind: BlueprintNodeErrorKind::ProductNotFound,
                ..
            })
        ),
        "reference to an unavailable product must be rejected, got {result:?}"
    );
}

#[test]
fn decoding_rejects_a_product_unsupported_by_its_buildable() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    // "other_product" exists in test_catalog but is not in test_machine's
    // production_targets, so this is a real catalog reference, just an
    // unsupported one for this specific buildable.
    value["nodes"][1]["production_target"] = serde_json::json!("other_product");
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert!(
        matches!(
            result,
            Err(BlueprintDocumentError::InvalidNode {
                kind: BlueprintNodeErrorKind::UnsupportedProduct,
                ..
            })
        ),
        "product unsupported by the buildable must be rejected, got {result:?}"
    );
}

#[test]
fn decoding_treats_catalog_id_mismatch_as_a_warning_not_an_error() {
    let source_catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(source_catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();

    let mut different_catalog_document: serde_json::Value =
        serde_json::from_slice(&encoded).unwrap();
    different_catalog_document["catalog_id"] = serde_json::json!("different_catalog");
    let tampered = serde_json::to_vec(&different_catalog_document).unwrap();

    let loaded = decode_blueprint_document(&tampered, source_catalog).unwrap();

    assert_eq!(
        loaded.compatibility,
        CatalogCompatibility::CatalogIdMismatch
    );
    assert_eq!(loaded.blueprint.nodes().len(), 2);
}

#[test]
fn decoding_maps_inverted_metadata_timestamps_to_the_domain_error() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    value["metadata"]["created_at"] = serde_json::json!("2024-01-02T00:00:00Z");
    value["metadata"]["updated_at"] = serde_json::json!("2024-01-01T00:00:00Z");
    let tampered = serde_json::to_vec(&value).unwrap();

    let result = decode_blueprint_document(&tampered, catalog);

    assert_eq!(
        result,
        Err(BlueprintDocumentError::InvalidMetadata(
            DocumentMetadataError::UpdatedBeforeCreated
        ))
    );
}

#[test]
fn encode_then_decode_round_trips_interfaces_exactly() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes_and_one_interface(catalog.clone());

    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let json = std::str::from_utf8(&encoded).unwrap();
    assert!(json.contains("\"interfaces\""));
    assert!(json.contains("\"Input\""));
    assert!(json.contains("\"west\""));

    let loaded = decode_blueprint_document(&encoded, catalog).unwrap();

    assert_eq!(loaded.blueprint.interfaces(), blueprint.interfaces());
}

#[test]
fn decoding_a_pre_existing_document_with_no_interfaces_key_yields_an_empty_list() {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    // Represents every blueprint file saved before this feature: no
    // "interfaces" key at all, not even an empty array.
    value
        .as_object_mut()
        .unwrap()
        .remove("interfaces")
        .expect("blueprint_with_two_nodes must encode an interfaces key to remove");
    let pre_existing = serde_json::to_vec(&value).unwrap();

    let loaded = decode_blueprint_document(&pre_existing, catalog).unwrap();

    assert!(loaded.blueprint.interfaces().is_empty());
}

#[test]
fn decoding_rejects_a_document_whose_interfaces_entry_has_a_blank_or_duplicate_name_or_an_off_boundary_anchor(
) {
    let catalog = test_catalog();
    let blueprint = blueprint_with_two_nodes_and_one_interface(catalog.clone());
    let encoded = encode_blueprint_document(&blueprint).unwrap();

    let mut blank_name: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    blank_name["interfaces"][0]["name"] = serde_json::json!("   ");
    let tampered = serde_json::to_vec(&blank_name).unwrap();
    let result = decode_blueprint_document(&tampered, catalog.clone());
    assert!(
        matches!(
            result,
            Err(BlueprintDocumentError::InvalidInterface {
                kind: factory_canvas::persistence::blueprint_document::BlueprintInterfaceErrorKind::BlankName,
                ..
            })
        ),
        "a blank interface name must be rejected, got {result:?}"
    );

    let mut off_boundary: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    off_boundary["interfaces"][0]["anchor"] = serde_json::json!({ "x": 1, "y": 1 });
    let tampered = serde_json::to_vec(&off_boundary).unwrap();
    let result = decode_blueprint_document(&tampered, catalog);
    assert!(
        matches!(
            result,
            Err(BlueprintDocumentError::InvalidInterface {
                kind: factory_canvas::persistence::blueprint_document::BlueprintInterfaceErrorKind::NotOnBoundary,
                ..
            })
        ),
        "an off-boundary interface anchor must be rejected, got {result:?}"
    );
}
