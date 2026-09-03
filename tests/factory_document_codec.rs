use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CategoryId, ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use factory_canvas::domain::document::{DocumentMetadata, DocumentMetadataError};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout};
use factory_canvas::persistence::factory_document::{
    decode_factory_document, encode_factory_document, CatalogCompatibility, FactoryDocumentError,
};
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
        )],
        vec![
            ProductDefinition::new(product_id, "Test Product"),
            ProductDefinition::new(ProductId::new("other_product").unwrap(), "Other Product"),
        ],
    )
    .unwrap()
}

fn document_json_with_entities(entities: serde_json::Value, next_entity_id: u64) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "catalog_id": "test_catalog",
        "catalog_data_version": "1.2.3",
        "metadata": {
            "name": "Test Factory",
            "description": null,
            "created_at": "2023-11-14T22:13:20Z",
            "updated_at": "2023-11-14T22:13:20Z"
        },
        "base_id": "test_base",
        "next_entity_id": next_entity_id,
        "entities": entities
    }))
    .unwrap()
}

#[test]
fn factory_v1_round_trip_preserves_layout_metadata_product_and_allocator() {
    let catalog = test_catalog();
    let mut layout = FactoryLayout::new(
        catalog.clone(),
        BaseId::new("test_base").expect("valid test base ID"),
    )
    .unwrap();
    let id = EntityId::new(7);
    layout
        .place(BlockInstance::new(
            id,
            BuildableId::new("test_machine").unwrap(),
            GridPoint::new(4, 5),
            Rotation::Clockwise90,
        ))
        .unwrap();
    layout
        .set_production_target(id, Some(ProductId::new("test_product").unwrap()))
        .unwrap();
    let created_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_123).unwrap();
    let metadata = DocumentMetadata::new(
        "Test Factory",
        Some("Round-trip fixture"),
        created_at,
        updated_at,
    )
    .unwrap();

    let encoded = encode_factory_document(&layout, Some(8), &metadata).unwrap();
    let json = std::str::from_utf8(&encoded).unwrap();
    assert!(json.contains("\"schema_version\": 1"));
    assert!(json.contains("\"rotation_degrees\": 90"));
    assert!(json.ends_with('\n'));

    let loaded = decode_factory_document(&encoded, catalog).unwrap();

    assert_eq!(loaded.layout, layout);
    assert_eq!(loaded.next_entity_id, Some(8));
    assert_eq!(loaded.metadata, metadata);
    assert_eq!(loaded.compatibility, CatalogCompatibility::Exact);
}

#[test]
fn missing_required_nullable_production_target_is_rejected() {
    let json = br#"{
        "schema_version": 1,
        "catalog_id": "test_catalog",
        "catalog_data_version": "1.2.3",
        "metadata": {
            "name": "Test Factory",
            "description": null,
            "created_at": "2023-11-14T22:13:20Z",
            "updated_at": "2023-11-14T22:13:20Z"
        },
        "base_id": "test_base",
        "next_entity_id": 2,
        "entities": [{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0
        }]
    }"#;

    let result = decode_factory_document(json, test_catalog());
    assert!(
        matches!(result, Err(FactoryDocumentError::InvalidJson { .. })),
        "missing nullable field returned {result:?}"
    );
}

#[test]
fn missing_required_nullable_next_entity_id_is_rejected() {
    let json = br#"{
        "schema_version": 1,
        "catalog_id": "test_catalog",
        "catalog_data_version": "1.2.3",
        "metadata": {
            "name": "Test Factory",
            "description": null,
            "created_at": "2023-11-14T22:13:20Z",
            "updated_at": "2023-11-14T22:13:20Z"
        },
        "base_id": "test_base",
        "entities": []
    }"#;

    let result = decode_factory_document(json, test_catalog());
    assert!(
        matches!(result, Err(FactoryDocumentError::InvalidJson { .. })),
        "missing nullable field returned {result:?}"
    );
}

#[test]
fn missing_required_nullable_description_is_rejected() {
    let json = br#"{
        "schema_version": 1,
        "catalog_id": "test_catalog",
        "catalog_data_version": "1.2.3",
        "metadata": {
            "name": "Test Factory",
            "created_at": "2023-11-14T22:13:20Z",
            "updated_at": "2023-11-14T22:13:20Z"
        },
        "base_id": "test_base",
        "next_entity_id": 1,
        "entities": []
    }"#;

    let result = decode_factory_document(json, test_catalog());
    assert!(
        matches!(result, Err(FactoryDocumentError::InvalidJson { .. })),
        "missing nullable field returned {result:?}"
    );
}

#[test]
fn entity_error_reports_the_original_json_position_after_deterministic_sorting() {
    let json = br#"{
        "schema_version": 1,
        "catalog_id": "test_catalog",
        "catalog_data_version": "1.2.3",
        "metadata": {
            "name": "Test Factory",
            "description": null,
            "created_at": "2023-11-14T22:13:20Z",
            "updated_at": "2023-11-14T22:13:20Z"
        },
        "base_id": "test_base",
        "next_entity_id": 3,
        "entities": [
            {
                "id": 2,
                "buildable_id": "test_machine",
                "origin": { "x": 5, "y": 5 },
                "rotation_degrees": 0,
                "production_target": null
            },
            {
                "id": 1,
                "buildable_id": "test_machine",
                "origin": { "x": 1, "y": 1 },
                "rotation_degrees": 45,
                "production_target": null
            }
        ]
    }"#;

    assert_eq!(
        decode_factory_document(json, test_catalog()),
        Err(FactoryDocumentError::InvalidRotation { entity_index: 1 })
    );
}

#[test]
fn missing_buildable_error_is_actionable_without_echoing_the_catalog_id() {
    let private_id = "private_buildable_sentinel";
    let json = format!(
        r#"{{
            "schema_version": 1,
            "catalog_id": "test_catalog",
            "catalog_data_version": "1.2.3",
            "metadata": {{
                "name": "Test Factory",
                "description": null,
                "created_at": "2023-11-14T22:13:20Z",
                "updated_at": "2023-11-14T22:13:20Z"
            }},
            "base_id": "test_base",
            "next_entity_id": 2,
            "entities": [{{
                "id": 1,
                "buildable_id": "{private_id}",
                "origin": {{ "x": 1, "y": 1 }},
                "rotation_degrees": 0,
                "production_target": null
            }}]
        }}"#
    );

    let error = decode_factory_document(json.as_bytes(), test_catalog()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "factory document entity 1 references an unavailable buildable"
    );
    assert!(!error.to_string().contains(private_id));
}

#[test]
fn missing_product_error_is_actionable_without_echoing_the_catalog_id() {
    let private_id = "private_product_sentinel";
    let json = document_json_with_entities(
        serde_json::json!([{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0,
            "production_target": private_id
        }]),
        2,
    );

    let error = decode_factory_document(&json, test_catalog()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "factory document entity 1 references an unavailable product"
    );
    assert!(!error.to_string().contains(private_id));
}

#[test]
fn unsupported_product_error_distinguishes_capability_from_missing_data() {
    let json = document_json_with_entities(
        serde_json::json!([{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0,
            "production_target": "other_product"
        }]),
        2,
    );

    let error = decode_factory_document(&json, test_catalog()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "factory document entity 1 selects a product unsupported by its buildable"
    );
}

#[test]
fn out_of_bounds_error_is_actionable_without_echoing_coordinates() {
    let json = document_json_with_entities(
        serde_json::json!([{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 29, "y": 29 },
            "rotation_degrees": 0,
            "production_target": null
        }]),
        2,
    );

    let error = decode_factory_document(&json, test_catalog()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "factory document entity 1 extends outside the selected base"
    );
    assert!(!error.to_string().contains("29"));
}

#[test]
fn collision_error_identifies_the_entity_without_echoing_the_other_id() {
    let json = document_json_with_entities(
        serde_json::json!([
            {
                "id": 1,
                "buildable_id": "test_machine",
                "origin": { "x": 1, "y": 1 },
                "rotation_degrees": 0,
                "production_target": null
            },
            {
                "id": 77,
                "buildable_id": "test_machine",
                "origin": { "x": 2, "y": 2 },
                "rotation_degrees": 0,
                "production_target": null
            }
        ]),
        78,
    );

    let error = decode_factory_document(&json, test_catalog()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "factory document entity 2 overlaps another entity"
    );
    assert!(!error.to_string().contains("77"));
}

#[test]
fn duplicate_id_error_is_distinct_from_physical_collision() {
    let json = document_json_with_entities(
        serde_json::json!([
            {
                "id": 7,
                "buildable_id": "test_machine",
                "origin": { "x": 1, "y": 1 },
                "rotation_degrees": 0,
                "production_target": null
            },
            {
                "id": 7,
                "buildable_id": "test_machine",
                "origin": { "x": 10, "y": 10 },
                "rotation_degrees": 0,
                "production_target": null
            }
        ]),
        8,
    );

    let error = decode_factory_document(&json, test_catalog()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "factory document entity 2 duplicates another entity ID"
    );
    assert!(!error.to_string().contains('7'));
}

#[test]
fn unsupported_schema_is_rejected_before_reading_version_specific_fields() {
    let private_version = 987_654_321_u64;
    let json = serde_json::to_vec(&serde_json::json!({
        "schema_version": private_version,
        "private_payload": "must_not_be_read"
    }))
    .unwrap();

    let error = decode_factory_document(&json, test_catalog()).unwrap_err();

    assert_eq!(error, FactoryDocumentError::UnsupportedSchemaVersion);
    assert!(!error.to_string().contains(&private_version.to_string()));
    assert!(!error.to_string().contains("private_payload"));
}

#[test]
fn unknown_fields_are_rejected_at_every_v1_object_boundary() {
    let base = document_json_with_entities(
        serde_json::json!([{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0,
            "production_target": null
        }]),
        2,
    );
    let base: serde_json::Value = serde_json::from_slice(&base).unwrap();
    let mut candidates = Vec::new();

    let mut top_level = base.clone();
    top_level["unexpected"] = serde_json::json!(true);
    candidates.push(top_level);

    let mut metadata = base.clone();
    metadata["metadata"]["unexpected"] = serde_json::json!(true);
    candidates.push(metadata);

    let mut entity = base.clone();
    entity["entities"][0]["unexpected"] = serde_json::json!(true);
    candidates.push(entity);

    let mut origin = base;
    origin["entities"][0]["origin"]["unexpected"] = serde_json::json!(true);
    candidates.push(origin);

    for candidate in candidates {
        let encoded = serde_json::to_vec(&candidate).unwrap();
        assert!(matches!(
            decode_factory_document(&encoded, test_catalog()),
            Err(FactoryDocumentError::InvalidJson { .. })
        ));
    }
}

#[test]
fn catalog_mismatches_warn_without_blocking_compatible_references() {
    let base = document_json_with_entities(serde_json::json!([]), 1);
    let base: serde_json::Value = serde_json::from_slice(&base).unwrap();
    let cases = [
        (
            "other_catalog",
            "1.2.3",
            CatalogCompatibility::CatalogIdMismatch,
        ),
        (
            "test_catalog",
            "9.8.7",
            CatalogCompatibility::DataVersionMismatch,
        ),
        (
            "other_catalog",
            "9.8.7",
            CatalogCompatibility::CatalogAndDataVersionMismatch,
        ),
    ];

    for (catalog_id, data_version, expected) in cases {
        let mut candidate = base.clone();
        candidate["catalog_id"] = serde_json::json!(catalog_id);
        candidate["catalog_data_version"] = serde_json::json!(data_version);
        let encoded = serde_json::to_vec(&candidate).unwrap();

        let loaded = decode_factory_document(&encoded, test_catalog()).unwrap();
        assert_eq!(loaded.compatibility, expected);
        assert!(loaded.layout.is_empty());
    }
}

#[test]
fn allocator_must_be_positive_and_greater_than_every_live_entity() {
    let empty_with_zero = document_json_with_entities(serde_json::json!([]), 0);
    assert_eq!(
        decode_factory_document(&empty_with_zero, test_catalog()),
        Err(FactoryDocumentError::InvalidNextEntityId)
    );

    let entity = serde_json::json!([{
        "id": 7,
        "buildable_id": "test_machine",
        "origin": { "x": 1, "y": 1 },
        "rotation_degrees": 0,
        "production_target": null
    }]);
    let not_greater = document_json_with_entities(entity.clone(), 7);
    assert_eq!(
        decode_factory_document(&not_greater, test_catalog()),
        Err(FactoryDocumentError::InvalidNextEntityId)
    );

    let mut exhausted: serde_json::Value =
        serde_json::from_slice(&document_json_with_entities(entity, 8)).unwrap();
    exhausted["next_entity_id"] = serde_json::Value::Null;
    let exhausted = serde_json::to_vec(&exhausted).unwrap();
    assert_eq!(
        decode_factory_document(&exhausted, test_catalog())
            .unwrap()
            .next_entity_id,
        None
    );
}

#[test]
fn encoder_emits_entities_in_id_order_with_all_rotation_values() {
    let catalog = test_catalog();
    let mut layout = FactoryLayout::new(catalog, BaseId::new("test_base").unwrap()).unwrap();
    let cases = [
        (4, GridPoint::new(15, 0), Rotation::Clockwise270),
        (2, GridPoint::new(5, 0), Rotation::Clockwise90),
        (1, GridPoint::new(0, 0), Rotation::Zero),
        (3, GridPoint::new(10, 0), Rotation::Clockwise180),
    ];
    for (id, origin, rotation) in cases {
        layout
            .place(BlockInstance::new(
                EntityId::new(id),
                BuildableId::new("test_machine").unwrap(),
                origin,
                rotation,
            ))
            .unwrap();
    }
    let timestamp = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let metadata = DocumentMetadata::new("Factory", None, timestamp, timestamp).unwrap();

    let encoded = encode_factory_document(&layout, Some(5), &metadata).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    let ids = value["entities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entity| entity["id"].as_u64().unwrap())
        .collect::<Vec<_>>();
    let rotations = value["entities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entity| entity["rotation_degrees"].as_u64().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec![1, 2, 3, 4]);
    assert_eq!(rotations, vec![0, 90, 180, 270]);
}

#[test]
fn malformed_json_reports_location_without_echoing_document_content() {
    let private_content = "private_payload_sentinel";
    let json = format!("{{\n  \"schema_version\": 1,\n  \"{private_content}\":");

    let error = decode_factory_document(json.as_bytes(), test_catalog()).unwrap_err();

    assert!(matches!(
        error,
        FactoryDocumentError::InvalidJson { line: 3, column: _ }
    ));
    assert!(!error.to_string().contains(private_content));
}

#[test]
fn invalid_scalar_values_are_classified_without_echoing_them() {
    let base = document_json_with_entities(
        serde_json::json!([{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0,
            "production_target": null
        }]),
        2,
    );
    let base: serde_json::Value = serde_json::from_slice(&base).unwrap();
    let cases = [
        (
            vec!["catalog_id"],
            serde_json::json!("Private Catalog!"),
            FactoryDocumentError::InvalidCatalogId,
            "Private Catalog!",
        ),
        (
            vec!["catalog_data_version"],
            serde_json::json!("private-version"),
            FactoryDocumentError::InvalidCatalogDataVersion,
            "private-version",
        ),
        (
            vec!["metadata", "created_at"],
            serde_json::json!("private-timestamp"),
            FactoryDocumentError::InvalidTimestamp,
            "private-timestamp",
        ),
        (
            vec!["base_id"],
            serde_json::json!("Private Base!"),
            FactoryDocumentError::InvalidBaseId,
            "Private Base!",
        ),
    ];

    for (path, value, expected, private_value) in cases {
        let mut candidate = base.clone();
        let mut target = &mut candidate;
        for segment in &path[..path.len() - 1] {
            target = &mut target[*segment];
        }
        target[path[path.len() - 1]] = value;
        let encoded = serde_json::to_vec(&candidate).unwrap();

        let error = decode_factory_document(&encoded, test_catalog()).unwrap_err();
        assert_eq!(error, expected);
        assert!(!error.to_string().contains(private_value));
    }
}

#[test]
fn encoder_rejects_an_allocator_that_could_reuse_a_live_id() {
    let catalog = test_catalog();
    let mut layout = FactoryLayout::new(catalog, BaseId::new("test_base").unwrap()).unwrap();
    layout
        .place(BlockInstance::new(
            EntityId::new(7),
            BuildableId::new("test_machine").unwrap(),
            GridPoint::new(1, 1),
            Rotation::Zero,
        ))
        .unwrap();
    let timestamp = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let metadata = DocumentMetadata::new("Factory", None, timestamp, timestamp).unwrap();

    assert_eq!(
        encode_factory_document(&layout, Some(7), &metadata),
        Err(FactoryDocumentError::InvalidNextEntityId)
    );
}

#[test]
fn schema_header_must_be_present_and_an_unsigned_integer() {
    let base: serde_json::Value =
        serde_json::from_slice(&document_json_with_entities(serde_json::json!([]), 1)).unwrap();
    let mut candidates = Vec::new();

    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("schema_version");
    candidates.push(missing);

    let mut string = base.clone();
    string["schema_version"] = serde_json::json!("1");
    candidates.push(string);

    let mut negative = base;
    negative["schema_version"] = serde_json::json!(-1);
    candidates.push(negative);

    for candidate in candidates {
        let encoded = serde_json::to_vec(&candidate).unwrap();
        assert!(matches!(
            decode_factory_document(&encoded, test_catalog()),
            Err(FactoryDocumentError::InvalidJson { .. })
        ));
    }
}

#[test]
fn zero_entity_id_is_rejected_on_decode() {
    let json = document_json_with_entities(
        serde_json::json!([{
            "id": 0,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0,
            "production_target": null
        }]),
        1,
    );

    assert_eq!(
        decode_factory_document(&json, test_catalog()),
        Err(FactoryDocumentError::InvalidEntityId { entity_index: 0 })
    );
}

#[test]
fn unavailable_base_is_rejected_without_echoing_its_id() {
    let private_id = "private_base_sentinel";
    let mut json: serde_json::Value =
        serde_json::from_slice(&document_json_with_entities(serde_json::json!([]), 1)).unwrap();
    json["base_id"] = serde_json::json!(private_id);

    let error =
        decode_factory_document(&serde_json::to_vec(&json).unwrap(), test_catalog()).unwrap_err();

    assert_eq!(error, FactoryDocumentError::InvalidBaseId);
    assert!(!error.to_string().contains(private_id));
}

#[test]
fn invalid_entity_identifiers_are_classified_without_echoing_them() {
    let base: serde_json::Value = serde_json::from_slice(&document_json_with_entities(
        serde_json::json!([{
            "id": 1,
            "buildable_id": "test_machine",
            "origin": { "x": 1, "y": 1 },
            "rotation_degrees": 0,
            "production_target": null
        }]),
        2,
    ))
    .unwrap();
    let cases = [
        (
            "buildable_id",
            "Private Buildable!",
            FactoryDocumentError::InvalidBuildableId { entity_index: 0 },
        ),
        (
            "production_target",
            "Private Product!",
            FactoryDocumentError::InvalidProductionTarget { entity_index: 0 },
        ),
    ];

    for (field, private_value, expected) in cases {
        let mut candidate = base.clone();
        candidate["entities"][0][field] = serde_json::json!(private_value);
        let error =
            decode_factory_document(&serde_json::to_vec(&candidate).unwrap(), test_catalog())
                .unwrap_err();

        assert_eq!(error, expected);
        assert!(!error.to_string().contains(private_value));
    }
}

#[test]
fn codec_maps_inverted_metadata_timestamps_to_the_domain_error() {
    let mut json: serde_json::Value =
        serde_json::from_slice(&document_json_with_entities(serde_json::json!([]), 1)).unwrap();
    json["metadata"]["created_at"] = serde_json::json!("2024-01-02T00:00:00Z");
    json["metadata"]["updated_at"] = serde_json::json!("2024-01-01T00:00:00Z");

    assert_eq!(
        decode_factory_document(&serde_json::to_vec(&json).unwrap(), test_catalog()),
        Err(FactoryDocumentError::InvalidMetadata(
            DocumentMetadataError::UpdatedBeforeCreated
        ))
    );
}
