use factory_canvas::domain::base::BaseTemplate;
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout, PlacementError};

#[test]
fn new_layout_starts_without_instances() {
    let layout = FactoryLayout::new(BaseTemplate::MainCurrent);

    assert!(layout.is_empty());
    assert_eq!(layout.len(), 0);
    assert_eq!(layout.instance(EntityId::new(1)), None);
}

#[test]
fn placement_rejects_duplicate_entity_id_without_replacing_original() {
    let id = EntityId::new(7);
    let original = BlockInstance::new(
        id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let duplicate = BlockInstance::new(
        id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(-1, -1),
        Rotation::Clockwise90,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);

    assert_eq!(layout.place(original), Ok(()));
    assert_eq!(
        layout.place(duplicate),
        Err(PlacementError::DuplicateEntityId { id })
    );
    assert_eq!(layout.len(), 1);
    assert_eq!(layout.instance(id), Some(&original));
}

#[test]
fn placement_enforces_half_open_base_bounds_atomically() {
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    let exact_fit_id = EntityId::new(10);
    let exact_fit = BlockInstance::new(
        exact_fit_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(78, 78),
        Rotation::Zero,
    );

    assert_eq!(layout.place(exact_fit), Ok(()));

    let out_of_bounds = [
        (EntityId::new(11), GridPoint::new(-1, 0)),
        (EntityId::new(12), GridPoint::new(0, -1)),
        (EntityId::new(13), GridPoint::new(79, 0)),
        (EntityId::new(14), GridPoint::new(0, 79)),
    ];

    for (id, origin) in out_of_bounds {
        let candidate = BlockInstance::new(
            id,
            BlockTemplate::XiranitePowerPole,
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
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let overlapping_id = EntityId::new(21);
    let overlapping = BlockInstance::new(
        overlapping_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(1, 1),
        Rotation::Clockwise180,
    );
    let touching_id = EntityId::new(22);
    let touching = BlockInstance::new(
        touching_id,
        BlockTemplate::CrushingUnit,
        GridPoint::new(2, 0),
        Rotation::Clockwise270,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);

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

    assert_eq!(layout.place(touching), Ok(()));
    assert_eq!(layout.instance(touching_id), Some(&touching));
    assert_eq!(layout.len(), 2);
}

#[test]
fn placement_reports_out_of_bounds_before_collision() {
    let existing_id = EntityId::new(30);
    let existing = BlockInstance::new(
        existing_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(78, 78),
        Rotation::Zero,
    );
    let candidate_id = EntityId::new(31);
    let candidate = BlockInstance::new(
        candidate_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(78, 78),
        Rotation::Clockwise90,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);

    assert_eq!(layout.place(existing), Ok(()));
    assert_eq!(
        layout.place(candidate),
        Err(PlacementError::OutOfBounds { id: candidate_id })
    );
    assert_eq!(layout.instance(candidate_id), None);
    assert_eq!(layout.instance(existing_id), Some(&existing));
    assert_eq!(layout.len(), 1);
}
