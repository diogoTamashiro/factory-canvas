mod support;

use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::catalog::BuildableId;
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout, PlacementError};

fn main_layout() -> FactoryLayout {
    let catalog = load_embedded_public_catalog().expect("public test catalog must load");
    let base_id = catalog.default_base_id().clone();
    FactoryLayout::new(catalog, base_id).expect("embedded default base exists")
}

#[test]
fn placement_resolves_defined_buildable_footprint_from_catalog() {
    let buildable_id = BuildableId::new("wide_machine").expect("valid test buildable ID");
    let instance = BlockInstance::new(
        EntityId::new(1),
        buildable_id.clone(),
        GridPoint::new(6, 5),
        Rotation::Zero,
    );
    let mut fitting_layout = support::layout_with_buildables(
        GridSize::new(13, 9).unwrap(),
        &[("wide_machine", GridSize::new(7, 4).unwrap())],
    );

    assert_eq!(fitting_layout.place(instance), Ok(()));

    let outside = BlockInstance::new(
        EntityId::new(2),
        buildable_id,
        GridPoint::new(7, 5),
        Rotation::Zero,
    );
    let mut bounded_layout = support::layout_with_buildables(
        GridSize::new(13, 9).unwrap(),
        &[("wide_machine", GridSize::new(7, 4).unwrap())],
    );

    assert_eq!(
        bounded_layout.place(outside),
        Err(PlacementError::OutOfBounds {
            id: EntityId::new(2)
        })
    );
}

#[test]
fn placement_rejects_unknown_buildable_id_without_mutating_layout() {
    let id = EntityId::new(9);
    let buildable_id = BuildableId::new("missing_machine").expect("valid missing buildable ID");
    let instance = BlockInstance::new(
        id,
        buildable_id.clone(),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let mut layout = main_layout();
    let before = layout.clone();

    assert_eq!(
        layout.place(instance),
        Err(PlacementError::BuildableNotFound { id, buildable_id })
    );
    assert_eq!(layout, before);
}

#[test]
fn resolved_instance_exposes_definition_and_effective_rotated_footprint() {
    let id = EntityId::new(10);
    let buildable_id = BuildableId::new("wide_machine").expect("valid test buildable ID");
    let instance = BlockInstance::new(
        id,
        buildable_id.clone(),
        GridPoint::new(1, 2),
        Rotation::Clockwise90,
    );
    let mut layout = support::layout_with_buildables(
        GridSize::new(13, 9).unwrap(),
        &[("wide_machine", GridSize::new(7, 4).unwrap())],
    );
    assert_eq!(layout.place(instance.clone()), Ok(()));

    let resolved = layout
        .resolved_instance(id)
        .expect("placed instance should resolve");

    assert_eq!(resolved.instance(), &instance);
    assert_eq!(resolved.definition().id(), &buildable_id);
    assert_eq!(resolved.effective_footprint(), GridSize::new(4, 7).unwrap());
}

#[test]
fn new_layout_starts_without_instances() {
    let layout = main_layout();

    assert!(layout.is_empty());
    assert_eq!(layout.len(), 0);
    assert_eq!(layout.instance(EntityId::new(1)), None);
}

#[test]
fn placement_rejects_duplicate_entity_id_without_replacing_original() {
    let id = EntityId::new(7);
    let original = BlockInstance::new(
        id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let duplicate = BlockInstance::new(
        id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(-1, -1),
        Rotation::Clockwise90,
    );
    let mut layout = main_layout();

    assert_eq!(layout.place(original.clone()), Ok(()));
    assert_eq!(
        layout.place(duplicate),
        Err(PlacementError::DuplicateEntityId { id })
    );
    assert_eq!(layout.len(), 1);
    assert_eq!(layout.instance(id), Some(&original));
}

#[test]
fn placement_enforces_half_open_base_bounds_atomically() {
    let mut layout = main_layout();
    let exact_fit_id = EntityId::new(10);
    let exact_fit = BlockInstance::new(
        exact_fit_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(78, 78),
        Rotation::Zero,
    );

    assert_eq!(layout.place(exact_fit.clone()), Ok(()));

    let out_of_bounds = [
        (EntityId::new(11), GridPoint::new(-1, 0)),
        (EntityId::new(12), GridPoint::new(0, -1)),
        (EntityId::new(13), GridPoint::new(79, 0)),
        (EntityId::new(14), GridPoint::new(0, 79)),
    ];

    for (id, origin) in out_of_bounds {
        let candidate = BlockInstance::new(
            id,
            support::buildable_id("xiranite_power_pole"),
            origin,
            Rotation::Clockwise90,
        );

        assert_eq!(
            layout.place(candidate),
            Err(PlacementError::OutOfBounds { id })
        );
        assert_eq!(layout.instance(id), None);
    }

    assert_eq!(layout.len(), 1);
    assert_eq!(layout.instance(exact_fit_id), Some(&exact_fit));
}

#[test]
fn placement_rejects_overlap_but_allows_edge_contact() {
    let pole_id = EntityId::new(20);
    let pole = BlockInstance::new(
        pole_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let overlapping_id = EntityId::new(21);
    let overlapping = BlockInstance::new(
        overlapping_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(1, 1),
        Rotation::Clockwise180,
    );
    let touching_id = EntityId::new(22);
    let touching = BlockInstance::new(
        touching_id,
        support::buildable_id("crushing_unit"),
        GridPoint::new(2, 0),
        Rotation::Clockwise270,
    );
    let mut layout = main_layout();

    assert_eq!(layout.place(pole), Ok(()));
    assert_eq!(
        layout.place(overlapping),
        Err(PlacementError::Collision {
            id: overlapping_id,
            conflicting_id: pole_id,
        })
    );
    assert_eq!(layout.instance(overlapping_id), None);
    assert_eq!(layout.len(), 1);

    assert_eq!(layout.place(touching.clone()), Ok(()));
    assert_eq!(layout.instance(touching_id), Some(&touching));
    assert_eq!(layout.len(), 2);
}

#[test]
fn placement_reports_out_of_bounds_before_collision() {
    let existing_id = EntityId::new(30);
    let existing = BlockInstance::new(
        existing_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(78, 78),
        Rotation::Zero,
    );
    let candidate_id = EntityId::new(31);
    let candidate = BlockInstance::new(
        candidate_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(78, 78),
        Rotation::Clockwise90,
    );
    let mut layout = main_layout();

    assert_eq!(layout.place(existing.clone()), Ok(()));
    assert_eq!(
        layout.place(candidate),
        Err(PlacementError::OutOfBounds { id: candidate_id })
    );
    assert_eq!(layout.instance(candidate_id), None);
    assert_eq!(layout.instance(existing_id), Some(&existing));
    assert_eq!(layout.len(), 1);
}
