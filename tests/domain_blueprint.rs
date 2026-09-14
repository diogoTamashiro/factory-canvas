use factory_canvas::domain::blueprint::{
    Blueprint, BlueprintCreationError, BlueprintEntityId, BlueprintId, BlueprintIdError,
    BlueprintInsertionError, BlueprintNodeInput, Interface, InterfaceError, Side,
};
use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CategoryId, ProductDefinition, ProductId, RegionDefinition, RegionId,
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

/// Shared catalog for `Blueprint::insert_into` tests (User Story 1):
/// one base (80x80), one buildable ("insertion_test_machine", 2x2
/// footprint), one product it supports. `version` lets tests build two
/// catalogs that differ only in data version (US1/AC5's "despite a
/// different catalog data version" scenario).
fn insertion_catalog_with_version(version: Version) -> Catalog {
    let region_id = RegionId::new("insertion_test_region").expect("valid test region ID");
    let base_id = BaseId::new("insertion_test_base").expect("valid test base ID");
    let category_id = CategoryId::new("insertion_test_category").expect("valid test category ID");
    Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("insertion_test_catalog").expect("valid test catalog ID"),
            version,
            "Insertion Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(
            region_id.clone(),
            "Insertion Test Region",
        )],
        vec![BaseDefinition::new(
            base_id,
            "Insertion Test Base",
            region_id,
            GridSize::new(80, 80).expect("positive test bounds"),
        )],
        vec![BuildableDefinition::new(
            insertion_buildable_id(),
            "Insertion Test Machine",
            category_id,
            "ITM",
            GridSize::new(2, 2).expect("positive footprint"),
            vec![insertion_product_id()],
        )],
        vec![ProductDefinition::new(
            insertion_product_id(),
            "Insertion Test Product",
        )],
    )
    .expect("valid insertion test catalog")
}

fn insertion_catalog() -> Catalog {
    insertion_catalog_with_version(Version::new(1, 0, 0))
}

/// Same base/region as `insertion_catalog`, but with zero buildables —
/// for US1/AC5's "buildable absent from the active catalog" branch.
fn insertion_catalog_without_buildable() -> Catalog {
    let region_id = RegionId::new("insertion_test_region").expect("valid test region ID");
    let base_id = BaseId::new("insertion_test_base").expect("valid test base ID");
    Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("insertion_test_catalog").expect("valid test catalog ID"),
            Version::new(1, 0, 0),
            "Insertion Test Catalog Without The Buildable",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(
            region_id.clone(),
            "Insertion Test Region",
        )],
        vec![BaseDefinition::new(
            base_id,
            "Insertion Test Base",
            region_id,
            GridSize::new(80, 80).expect("positive test bounds"),
        )],
        Vec::new(),
        Vec::new(),
    )
    .expect("valid insertion test catalog without the buildable")
}

fn insertion_buildable_id() -> BuildableId {
    support::buildable_id("insertion_test_machine")
}

fn insertion_product_id() -> ProductId {
    ProductId::new("insertion_test_product").expect("valid test product ID")
}

fn empty_insertion_layout() -> FactoryLayout {
    let catalog = insertion_catalog();
    let base_id = catalog.default_base_id().clone();
    FactoryLayout::new(catalog, base_id).expect("insertion test base must exist")
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
        Vec::new(),
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
        Vec::new(),
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
        Vec::new(),
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
        Vec::new(),
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
        Vec::new(),
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
        Vec::new(),
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
        Vec::new(),
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

#[test]
fn insert_into_places_every_node_at_its_relative_offset_with_fresh_sequential_ids() {
    let catalog = insertion_catalog();
    let mut source = FactoryLayout::new(catalog.clone(), catalog.default_base_id().clone())
        .expect("insertion test base must exist");
    let first_source_id = EntityId::new(5);
    let second_source_id = EntityId::new(7);
    source
        .place(BlockInstance::new(
            first_source_id,
            insertion_buildable_id(),
            GridPoint::new(2, 3),
            Rotation::Zero,
        ))
        .expect("first source instance must be placeable");
    source
        .place(BlockInstance::new(
            second_source_id,
            insertion_buildable_id(),
            GridPoint::new(10, 3),
            Rotation::Zero,
        ))
        .expect("second source instance must be placeable");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![first_source_id, second_source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection of two placed entities must produce a blueprint");

    let mut destination = empty_insertion_layout();

    let next_id = blueprint
        .insert_into(&mut destination, GridPoint::new(20, 20), 1000)
        .expect("insertion into an empty, in-bounds destination must succeed");

    assert_eq!(next_id, 1002);
    assert_eq!(destination.len(), 2);
    let first = destination
        .instance(EntityId::new(1000))
        .expect("first inserted node must get the first fresh ID");
    assert_eq!(first.origin(), GridPoint::new(20, 20));
    assert_eq!(first.buildable_id(), &insertion_buildable_id());
    let second = destination
        .instance(EntityId::new(1001))
        .expect("second inserted node must get the next fresh ID");
    assert_eq!(second.origin(), GridPoint::new(28, 20));
}

#[test]
fn insert_into_preserves_buildable_rotation_and_configured_product_per_node() {
    let catalog = insertion_catalog();
    let mut source = FactoryLayout::new(catalog.clone(), catalog.default_base_id().clone())
        .expect("insertion test base must exist");
    let source_id = EntityId::new(4);
    source
        .place(BlockInstance::new(
            source_id,
            insertion_buildable_id(),
            GridPoint::new(5, 5),
            Rotation::Clockwise90,
        ))
        .expect("source instance must be placeable");
    source
        .set_production_target(source_id, Some(insertion_product_id()))
        .expect("configuring a supported product must succeed");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection must produce a blueprint");

    let mut destination = empty_insertion_layout();

    blueprint
        .insert_into(&mut destination, GridPoint::new(30, 30), 1)
        .expect("insertion into an empty, in-bounds destination must succeed");

    let inserted = destination
        .instance(EntityId::new(1))
        .expect("inserted instance must exist under the first fresh ID");
    assert_eq!(inserted.buildable_id(), &insertion_buildable_id());
    assert_eq!(inserted.rotation(), Rotation::Clockwise90);
    assert_eq!(inserted.production_target(), Some(&insertion_product_id()));
}

#[test]
fn insert_into_out_of_bounds_leaves_the_layout_completely_unchanged() {
    let catalog = insertion_catalog();
    let mut source = FactoryLayout::new(catalog.clone(), catalog.default_base_id().clone())
        .expect("insertion test base must exist");
    let source_id = EntityId::new(1);
    source
        .place(BlockInstance::new(
            source_id,
            insertion_buildable_id(),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .expect("source instance must be placeable");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection must produce a blueprint");

    let mut destination = empty_insertion_layout();
    let before = destination.clone();

    // Base bounds are 80x80; a 2x2 footprint at (79, 79) would occupy
    // (79, 79)-(81, 81), extending past the right/bottom edge.
    let result = blueprint.insert_into(&mut destination, GridPoint::new(79, 79), 1000);

    assert_eq!(
        result,
        Err(BlueprintInsertionError::OutOfBounds { node_index: 0 })
    );
    assert_eq!(destination, before);
}

#[test]
fn insert_into_colliding_with_an_existing_instance_leaves_the_layout_completely_unchanged() {
    let catalog = insertion_catalog();
    let mut source = FactoryLayout::new(catalog.clone(), catalog.default_base_id().clone())
        .expect("insertion test base must exist");
    let source_id = EntityId::new(1);
    source
        .place(BlockInstance::new(
            source_id,
            insertion_buildable_id(),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .expect("source instance must be placeable");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection must produce a blueprint");

    let mut destination = empty_insertion_layout();
    destination
        .place(BlockInstance::new(
            EntityId::new(1),
            insertion_buildable_id(),
            GridPoint::new(10, 10),
            Rotation::Zero,
        ))
        .expect("existing destination instance must be placeable");
    let before = destination.clone();

    let result = blueprint.insert_into(&mut destination, GridPoint::new(10, 10), 1000);

    assert_eq!(
        result,
        Err(BlueprintInsertionError::Collision {
            node_index: 0,
            conflicting_id: EntityId::new(1),
        })
    );
    assert_eq!(destination, before);
}

#[test]
fn insert_into_colliding_between_two_of_its_own_nodes_is_rejected() {
    let catalog = insertion_catalog();
    let nodes = vec![
        BlueprintNodeInput {
            id: BlueprintEntityId::new(1),
            buildable_id: insertion_buildable_id(),
            relative_origin: GridPoint::new(0, 0),
            rotation: Rotation::Zero,
            production_target: None,
        },
        BlueprintNodeInput {
            id: BlueprintEntityId::new(2),
            buildable_id: insertion_buildable_id(),
            relative_origin: GridPoint::new(1, 0),
            rotation: Rotation::Zero,
            production_target: None,
        },
    ];
    let blueprint = Blueprint::from_nodes(
        test_blueprint_id(),
        catalog,
        test_metadata(),
        nodes,
        Vec::new(),
    )
    .expect(
        "both nodes reference a valid buildable, so construction must succeed \
         even though their footprints overlap — insert_into, not from_nodes, \
         is what must reject the overlap",
    );

    let mut destination = empty_insertion_layout();
    let before = destination.clone();

    let result = blueprint.insert_into(&mut destination, GridPoint::new(20, 20), 1000);

    assert_eq!(
        result,
        Err(BlueprintInsertionError::Collision {
            node_index: 1,
            conflicting_id: EntityId::new(1000),
        })
    );
    assert_eq!(destination, before);
}

#[test]
fn insert_into_rejects_a_node_whose_buildable_is_absent_from_the_active_catalog() {
    let source_catalog = insertion_catalog();
    let mut source = FactoryLayout::new(
        source_catalog.clone(),
        source_catalog.default_base_id().clone(),
    )
    .expect("insertion test base must exist");
    let source_id = EntityId::new(1);
    source
        .place(BlockInstance::new(
            source_id,
            insertion_buildable_id(),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .expect("source instance must be placeable");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection must produce a blueprint");

    let destination_catalog = insertion_catalog_without_buildable();
    let mut destination = FactoryLayout::new(
        destination_catalog.clone(),
        destination_catalog.default_base_id().clone(),
    )
    .expect("insertion test base must exist even without the buildable");
    let before = destination.clone();

    let result = blueprint.insert_into(&mut destination, GridPoint::new(10, 10), 1000);

    assert_eq!(
        result,
        Err(BlueprintInsertionError::BuildableNotFound {
            node_index: 0,
            buildable_id: insertion_buildable_id(),
        })
    );
    assert_eq!(destination, before);
}

#[test]
fn insert_into_succeeds_when_every_reference_still_resolves_despite_a_different_catalog_data_version(
) {
    let source_catalog = insertion_catalog_with_version(Version::new(1, 0, 0));
    let mut source = FactoryLayout::new(
        source_catalog.clone(),
        source_catalog.default_base_id().clone(),
    )
    .expect("insertion test base must exist");
    let source_id = EntityId::new(1);
    source
        .place(BlockInstance::new(
            source_id,
            insertion_buildable_id(),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .expect("source instance must be placeable");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection must produce a blueprint");
    assert_eq!(
        blueprint.provenance().data_version(),
        &Version::new(1, 0, 0)
    );

    let destination_catalog = insertion_catalog_with_version(Version::new(2, 0, 0));
    let mut destination = FactoryLayout::new(
        destination_catalog.clone(),
        destination_catalog.default_base_id().clone(),
    )
    .expect("insertion test base must exist");

    let result = blueprint.insert_into(&mut destination, GridPoint::new(10, 10), 1);

    assert_eq!(
        result,
        Ok(2),
        "the buildable and product IDs still resolve in the destination catalog, \
         so insertion must succeed even though its data version differs from \
         the blueprint's recorded provenance"
    );
    assert_eq!(destination.len(), 1);
}

#[test]
fn insert_into_rejects_entity_id_allocator_exhaustion_before_placing_anything() {
    let catalog = insertion_catalog();
    let nodes = vec![BlueprintNodeInput {
        id: BlueprintEntityId::new(1),
        buildable_id: insertion_buildable_id(),
        relative_origin: GridPoint::new(0, 0),
        rotation: Rotation::Zero,
        production_target: None,
    }];
    let blueprint = Blueprint::from_nodes(
        test_blueprint_id(),
        catalog,
        test_metadata(),
        nodes,
        Vec::new(),
    )
    .expect("single valid node must produce a blueprint");

    let mut destination = empty_insertion_layout();
    let before = destination.clone();

    let result = blueprint.insert_into(&mut destination, GridPoint::new(0, 0), u64::MAX);

    assert_eq!(result, Err(BlueprintInsertionError::EntityIdsExhausted));
    assert_eq!(destination, before);
}

#[test]
fn insert_into_rejects_a_node_whose_translated_coordinate_would_overflow() {
    let catalog = insertion_catalog();
    let nodes = vec![BlueprintNodeInput {
        id: BlueprintEntityId::new(1),
        buildable_id: insertion_buildable_id(),
        relative_origin: GridPoint::new(1, 0),
        rotation: Rotation::Zero,
        production_target: None,
    }];
    let blueprint = Blueprint::from_nodes(
        test_blueprint_id(),
        catalog,
        test_metadata(),
        nodes,
        Vec::new(),
    )
    .expect("single valid node must produce a blueprint");

    let mut destination = empty_insertion_layout();
    let before = destination.clone();

    let result = blueprint.insert_into(&mut destination, GridPoint::new(i32::MAX, 0), 1);

    assert_eq!(
        result,
        Err(BlueprintInsertionError::CoordinateOverflow { node_index: 0 })
    );
    assert_eq!(destination, before);
}

#[test]
fn inserted_instances_are_individually_movable_rotatable_and_removable_with_no_group_link() {
    let catalog = insertion_catalog();
    let mut source = FactoryLayout::new(catalog.clone(), catalog.default_base_id().clone())
        .expect("insertion test base must exist");
    let first_source_id = EntityId::new(1);
    let second_source_id = EntityId::new(2);
    source
        .place(BlockInstance::new(
            first_source_id,
            insertion_buildable_id(),
            GridPoint::new(0, 0),
            Rotation::Zero,
        ))
        .expect("first source instance must be placeable");
    source
        .place(BlockInstance::new(
            second_source_id,
            insertion_buildable_id(),
            GridPoint::new(10, 0),
            Rotation::Zero,
        ))
        .expect("second source instance must be placeable");
    let blueprint = Blueprint::from_selection(
        &source,
        vec![first_source_id, second_source_id],
        test_blueprint_id(),
        test_metadata(),
        Vec::new(),
    )
    .expect("selection of two placed entities must produce a blueprint");

    let mut destination = empty_insertion_layout();
    blueprint
        .insert_into(&mut destination, GridPoint::new(30, 30), 1000)
        .expect("insertion into an empty, in-bounds destination must succeed");

    let first_id = EntityId::new(1000);
    let second_id = EntityId::new(1001);

    // Moving the first inserted instance must not move, rotate, or
    // otherwise affect the second — there is no persisted group link.
    destination
        .move_instance(first_id, GridPoint::new(50, 50))
        .expect("first inserted instance must be independently movable");
    assert_eq!(
        destination.instance(second_id).unwrap().origin(),
        GridPoint::new(40, 30),
        "moving the first inserted instance must not move the second"
    );

    destination
        .rotate_instance(second_id, Rotation::Clockwise90)
        .expect("second inserted instance must be independently rotatable");
    assert_eq!(
        destination.instance(first_id).unwrap().rotation(),
        Rotation::Zero,
        "rotating the second inserted instance must not rotate the first"
    );

    destination.remove_instance(first_id);
    assert!(
        destination.instance(first_id).is_none(),
        "first inserted instance must be independently removable"
    );
    assert!(
        destination.instance(second_id).is_some(),
        "removing the first inserted instance must not remove the second"
    );
}

/// A single 3x3-footprint instance at (5, 5), captured with no interfaces
/// unless the caller supplies some — gives `Interface` tests a real
/// interior tile ((1, 1)) as well as four true corners, which a 2x2
/// footprint (this file's other helpers) cannot: every tile of a 2x2
/// footprint is itself a corner.
fn blueprint_with_interfaces(
    interfaces: Vec<Interface>,
) -> Result<Blueprint, BlueprintCreationError> {
    let (mut layout, _product_id) = layout_with_a_product_capable_buildable();
    let entity_id = EntityId::new(1);
    layout
        .place(BlockInstance::new(
            entity_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(5, 5),
            Rotation::Zero,
        ))
        .expect("entity must be placeable");

    Blueprint::from_selection(
        &layout,
        vec![entity_id],
        test_blueprint_id(),
        test_metadata(),
        interfaces,
    )
}

#[test]
fn from_selection_with_one_valid_interface_persists_its_name_and_location() {
    let interface = Interface::new("Input", GridPoint::new(0, 0), Side::West);

    let blueprint = blueprint_with_interfaces(vec![interface.clone()])
        .expect("a single valid interface must not be rejected");

    assert_eq!(blueprint.interfaces(), &[interface]);
}

#[test]
fn from_selection_with_zero_interfaces_remains_a_fully_valid_blueprint() {
    let blueprint =
        blueprint_with_interfaces(Vec::new()).expect("zero interfaces must remain valid");

    assert!(blueprint.interfaces().is_empty());
}

#[test]
fn from_selection_rejects_two_interfaces_sharing_a_trimmed_name() {
    let result = blueprint_with_interfaces(vec![
        Interface::new("Input", GridPoint::new(0, 0), Side::West),
        Interface::new(" Input ", GridPoint::new(0, 2), Side::West),
    ]);

    assert_eq!(
        result,
        Err(BlueprintCreationError::InvalidInterface(
            InterfaceError::DuplicateName { index: 1 }
        ))
    );
}

#[test]
fn from_selection_rejects_a_blank_or_whitespace_only_interface_name() {
    let result = blueprint_with_interfaces(vec![Interface::new(
        "   ",
        GridPoint::new(0, 0),
        Side::West,
    )]);

    assert_eq!(
        result,
        Err(BlueprintCreationError::InvalidInterface(
            InterfaceError::BlankName { index: 0 }
        ))
    );
}

#[test]
fn from_selection_rejects_an_interface_anchor_side_not_on_the_bounding_rectangle_boundary() {
    // The bounding rectangle is (0, 0)-(3, 3): a true interior tile at
    // (1, 1) cannot satisfy any side.
    let interior = blueprint_with_interfaces(vec![Interface::new(
        "Interior",
        GridPoint::new(1, 1),
        Side::North,
    )]);
    assert_eq!(
        interior,
        Err(BlueprintCreationError::InvalidInterface(
            InterfaceError::NotOnBoundary { index: 0 }
        ))
    );

    // (1, 0) is on the top edge (North is valid there), but South does
    // not point outward at that tile.
    let wrong_side = blueprint_with_interfaces(vec![Interface::new(
        "WrongSide",
        GridPoint::new(1, 0),
        Side::South,
    )]);
    assert_eq!(
        wrong_side,
        Err(BlueprintCreationError::InvalidInterface(
            InterfaceError::NotOnBoundary { index: 0 }
        ))
    );
}

#[test]
fn from_selection_accepts_a_corner_anchor_with_either_of_its_two_outward_sides() {
    let west = blueprint_with_interfaces(vec![Interface::new(
        "West",
        GridPoint::new(0, 0),
        Side::West,
    )]);
    assert!(
        west.is_ok(),
        "the top-left corner's West side must be a valid boundary point"
    );

    let north = blueprint_with_interfaces(vec![Interface::new(
        "North",
        GridPoint::new(0, 0),
        Side::North,
    )]);
    assert!(
        north.is_ok(),
        "the top-left corner's North side must also be a valid boundary point"
    );
}
