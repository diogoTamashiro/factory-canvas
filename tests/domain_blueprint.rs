use factory_canvas::domain::blueprint::{
    Blueprint, BlueprintCreationError, BlueprintEntityId, BlueprintId, BlueprintIdError,
};
use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, Catalog, CatalogId, CatalogMetadata, CategoryId,
    ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use factory_canvas::domain::document::DocumentMetadata;
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout};
use semver::Version;
use time::OffsetDateTime;

mod support;

fn fixed_timestamp() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("test timestamp must be valid")
}

fn test_blueprint_id() -> BlueprintId {
    BlueprintId::parse("blueprint_00000000000000000000000000000001")
        .expect("test blueprint ID must be valid")
}

fn test_metadata() -> DocumentMetadata {
    DocumentMetadata::new("Test Blueprint", None, fixed_timestamp(), fixed_timestamp())
        .expect("test blueprint metadata must be valid")
}

#[test]
fn selection_becomes_relative_blueprint_in_source_entity_order() {
    let mut layout = support::layout_with_buildables(
        GridSize::new(80, 80).expect("positive test bounds"),
        &[
            (
                "xiranite_power_pole",
                GridSize::new(2, 2).expect("positive footprint"),
            ),
            (
                "refinery_unit",
                GridSize::new(3, 3).expect("positive footprint"),
            ),
        ],
    );
    let pole_id = EntityId::new(9);
    let refinery_id = EntityId::new(3);
    layout
        .place(BlockInstance::new(
            pole_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(10, 8),
            Rotation::Clockwise90,
        ))
        .expect("pole must be placeable");
    layout
        .place(BlockInstance::new(
            refinery_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(6, 4),
            Rotation::Zero,
        ))
        .expect("refinery must be placeable");
    let blueprint_id = test_blueprint_id();
    let metadata = test_metadata();

    let blueprint = Blueprint::from_selection(
        &layout,
        vec![pole_id, refinery_id],
        blueprint_id.clone(),
        metadata.clone(),
    )
    .expect("selection of two placed entities must produce a blueprint");

    assert_eq!(blueprint.id(), &blueprint_id);
    assert_eq!(blueprint.metadata(), &metadata);
    assert_eq!(blueprint.nodes().len(), 2);

    // Source entity order is ascending by original EntityId: refinery (3) before pole (9),
    // regardless of the order entities were passed into from_selection.
    assert_eq!(blueprint.nodes()[0].id(), BlueprintEntityId::new(1));
    assert_eq!(
        blueprint.nodes()[0].buildable_id(),
        &support::buildable_id("refinery_unit")
    );
    assert_eq!(blueprint.nodes()[1].id(), BlueprintEntityId::new(2));
    assert_eq!(
        blueprint.nodes()[1].buildable_id(),
        &support::buildable_id("xiranite_power_pole")
    );

    // Normalized relative to the minimum x/y among selected origins: (6, 4).
    assert_eq!(blueprint.nodes()[0].relative_origin(), GridPoint::new(0, 0));
    assert_eq!(blueprint.nodes()[1].relative_origin(), GridPoint::new(4, 4));
    assert_eq!(blueprint.nodes()[0].rotation(), Rotation::Zero);
    assert_eq!(blueprint.nodes()[1].rotation(), Rotation::Clockwise90);
}

#[test]
fn from_selection_rejects_an_empty_selection() {
    let layout = support::layout_with_buildables(
        GridSize::new(80, 80).expect("positive test bounds"),
        &[(
            "xiranite_power_pole",
            GridSize::new(2, 2).expect("positive footprint"),
        )],
    );

    let result = Blueprint::from_selection(
        &layout,
        Vec::<EntityId>::new(),
        test_blueprint_id(),
        test_metadata(),
    );

    assert_eq!(result, Err(BlueprintCreationError::EmptySelection));
}

#[test]
fn from_selection_rejects_an_id_absent_from_the_layout() {
    let mut layout = support::layout_with_buildables(
        GridSize::new(80, 80).expect("positive test bounds"),
        &[(
            "xiranite_power_pole",
            GridSize::new(2, 2).expect("positive footprint"),
        )],
    );
    let placed_id = EntityId::new(1);
    layout
        .place(BlockInstance::new(
            placed_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .expect("entity must be placeable");
    let missing_id = EntityId::new(99);

    let result = Blueprint::from_selection(
        &layout,
        vec![placed_id, missing_id],
        test_blueprint_id(),
        test_metadata(),
    );

    assert_eq!(
        result,
        Err(BlueprintCreationError::EntityNotFound { id: missing_id })
    );
}

#[test]
fn from_selection_deduplicates_repeated_ids_deterministically() {
    let mut layout = support::layout_with_buildables(
        GridSize::new(80, 80).expect("positive test bounds"),
        &[(
            "xiranite_power_pole",
            GridSize::new(2, 2).expect("positive footprint"),
        )],
    );
    let entity_id = EntityId::new(5);
    layout
        .place(BlockInstance::new(
            entity_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(3, 3),
            Rotation::Zero,
        ))
        .expect("entity must be placeable");

    let blueprint = Blueprint::from_selection(
        &layout,
        vec![entity_id, entity_id, entity_id],
        test_blueprint_id(),
        test_metadata(),
    )
    .expect("repeated IDs for the same entity must still produce a blueprint");

    assert_eq!(blueprint.nodes().len(), 1);
    assert_eq!(blueprint.nodes()[0].id(), BlueprintEntityId::new(1));
}

#[test]
fn from_selection_preserves_origin_rotation_buildable_id_and_is_independent_of_later_layout_mutation(
) {
    let mut layout = support::layout_with_buildables(
        GridSize::new(80, 80).expect("positive test bounds"),
        &[(
            "refinery_unit",
            GridSize::new(3, 3).expect("positive footprint"),
        )],
    );
    let entity_id = EntityId::new(1);
    layout
        .place(BlockInstance::new(
            entity_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(2, 2),
            Rotation::Clockwise180,
        ))
        .expect("entity must be placeable");

    let blueprint = Blueprint::from_selection(
        &layout,
        vec![entity_id],
        test_blueprint_id(),
        test_metadata(),
    )
    .expect("selection must produce a blueprint");

    // Mutate the source factory after capture: move, remove, and re-place under a new ID.
    layout
        .move_instances_by(&[entity_id], GridPoint::new(8, 8))
        .expect("move must succeed");
    layout.remove_instance(entity_id);
    layout
        .place(BlockInstance::new(
            EntityId::new(2),
            support::buildable_id("refinery_unit"),
            GridPoint::new(20, 20),
            Rotation::Zero,
        ))
        .expect("replacement entity must be placeable");

    assert_eq!(blueprint.nodes().len(), 1);
    assert_eq!(blueprint.nodes()[0].id(), BlueprintEntityId::new(1));
    assert_eq!(blueprint.nodes()[0].relative_origin(), GridPoint::new(0, 0));
    assert_eq!(blueprint.nodes()[0].rotation(), Rotation::Clockwise180);
    assert_eq!(
        blueprint.nodes()[0].buildable_id(),
        &support::buildable_id("refinery_unit")
    );
}

fn layout_with_a_product_capable_buildable() -> (FactoryLayout, ProductId) {
    let region_id = RegionId::new("blueprint_test_region").expect("valid region ID");
    let base_id = BaseId::new("blueprint_test_base").expect("valid base ID");
    let category_id = CategoryId::new("blueprint_test_category").expect("valid category ID");
    let product_id = ProductId::new("blueprint_test_product").expect("valid product ID");
    let catalog = Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("blueprint_test_catalog").expect("valid catalog ID"),
            Version::new(1, 0, 0),
            "Blueprint Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(
            region_id.clone(),
            "Blueprint Test Region",
        )],
        vec![BaseDefinition::new(
            base_id.clone(),
            "Blueprint Test Base",
            region_id,
            GridSize::new(80, 80).expect("positive test bounds"),
        )],
        vec![BuildableDefinition::new(
            support::buildable_id("refinery_unit"),
            "Refinery Unit",
            category_id,
            "RU",
            GridSize::new(3, 3).expect("positive footprint"),
            vec![product_id.clone()],
        )],
        vec![ProductDefinition::new(
            product_id.clone(),
            "Blueprint Test Product",
        )],
    )
    .expect("valid test catalog");

    (
        FactoryLayout::new(catalog, base_id).expect("test base must exist"),
        product_id,
    )
}

#[test]
fn from_selection_preserves_a_configured_production_target() {
    let (mut layout, product_id) = layout_with_a_product_capable_buildable();
    let entity_id = EntityId::new(4);
    layout
        .place(BlockInstance::new(
            entity_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(5, 5),
            Rotation::Zero,
        ))
        .expect("entity must be placeable");
    layout
        .set_production_target(entity_id, Some(product_id.clone()))
        .expect("configuring a supported product must succeed");

    let blueprint = Blueprint::from_selection(
        &layout,
        vec![entity_id],
        test_blueprint_id(),
        test_metadata(),
    )
    .expect("selection must produce a blueprint");

    assert_eq!(blueprint.nodes()[0].production_target(), Some(&product_id));
}

#[test]
fn from_selection_preserves_the_absence_of_a_production_target() {
    let (mut layout, _product_id) = layout_with_a_product_capable_buildable();
    let entity_id = EntityId::new(4);
    layout
        .place(BlockInstance::new(
            entity_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(5, 5),
            Rotation::Zero,
        ))
        .expect("entity must be placeable");

    let blueprint = Blueprint::from_selection(
        &layout,
        vec![entity_id],
        test_blueprint_id(),
        test_metadata(),
    )
    .expect("selection must produce a blueprint");

    assert_eq!(blueprint.nodes()[0].production_target(), None);
}

#[test]
fn blueprint_id_generates_and_round_trips_the_simple_uuid_format() {
    let generated = BlueprintId::generate();

    let rendered = generated.as_str();
    assert!(rendered.starts_with("blueprint_"));
    let hex = rendered.strip_prefix("blueprint_").unwrap();
    assert_eq!(hex.len(), 32);
    assert!(hex.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(hex.to_ascii_lowercase(), hex);

    let parsed = BlueprintId::parse(rendered).expect("generated ID must parse back");
    assert_eq!(parsed, generated);
}

#[test]
fn blueprint_id_parse_rejects_missing_prefix_and_invalid_uuid() {
    assert_eq!(
        BlueprintId::parse("00000000000000000000000000000001"),
        Err(BlueprintIdError::MissingPrefix)
    );
    assert_eq!(
        BlueprintId::parse("blueprint_not-a-uuid"),
        Err(BlueprintIdError::InvalidUuid)
    );

    // "not-a-uuid" is both wrong-length and non-hex, so it never actually
    // exercises the charset check. Build a dedicated exactly-32-character
    // string with exactly one non-hex byte, so the non-hex rejection path is
    // exercised independently of the length check.
    let one_invalid_char_hex = "0".repeat(31) + "g";
    assert_eq!(one_invalid_char_hex.len(), 32);
    let one_invalid_char = format!("blueprint_{one_invalid_char_hex}");
    assert_eq!(
        BlueprintId::parse(&one_invalid_char),
        Err(BlueprintIdError::InvalidUuid),
        "a single non-hex byte in an otherwise-32-character string must be rejected"
    );

    // Build the too-short case programmatically (31 hex digits, one short of 32)
    // so the invariant is enforced by construction rather than by hand-counting
    // a literal.
    let too_short_hex = "0".repeat(31);
    assert_eq!(too_short_hex.len(), 31);
    let too_short = format!("blueprint_{too_short_hex}");
    assert_eq!(
        BlueprintId::parse(&too_short),
        Err(BlueprintIdError::InvalidUuid)
    );

    // Build a valid lowercase 32-hex baseline, confirm it round-trips, then derive
    // the uppercase case via an actual transformation of that exact string (not an
    // independently retyped literal) so the two strings are provably identical
    // except for case.
    let valid_lowercase_hex = "0".repeat(29) + "abc";
    assert_eq!(valid_lowercase_hex.len(), 32);
    assert!(BlueprintId::parse(&format!("blueprint_{valid_lowercase_hex}")).is_ok());
    let uppercase_hex = valid_lowercase_hex.to_ascii_uppercase();
    assert_eq!(uppercase_hex.len(), 32);
    assert_ne!(
        uppercase_hex, valid_lowercase_hex,
        "the transformation must actually change something, or this case would be indistinguishable from the lowercase baseline"
    );
    let uppercase = format!("blueprint_{uppercase_hex}");
    assert_eq!(
        BlueprintId::parse(&uppercase),
        Err(BlueprintIdError::InvalidUuid),
        "uppercase hex digits must be rejected even though the lowercase equivalent is valid and exactly 32 characters"
    );
}
