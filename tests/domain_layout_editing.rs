mod support;

use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::catalog::BuildableId;
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout, InstanceEditError};

fn main_layout() -> FactoryLayout {
    let catalog = load_embedded_public_catalog().expect("public test catalog must load");
    let base_id = catalog.default_base_id().clone();
    FactoryLayout::new(catalog, base_id).expect("public default base must exist")
}

#[test]
fn editing_uses_runtime_footprint_for_rotation_and_edge_move() {
    let id = EntityId::new(4);
    let buildable_id = BuildableId::new("wide_machine").expect("valid test buildable ID");
    let mut layout = support::layout_with_buildables(
        GridSize::new(13, 13).unwrap(),
        &[("wide_machine", GridSize::new(7, 4).unwrap())],
    );
    assert_eq!(
        layout.place(BlockInstance::new(
            id,
            buildable_id,
            GridPoint::new(1, 1),
            Rotation::Zero,
        )),
        Ok(())
    );

    assert_eq!(layout.rotate_instance(id, Rotation::Clockwise90), Ok(()));
    assert_eq!(layout.move_instance(id, GridPoint::new(9, 6)), Ok(()));

    let resolved = layout
        .resolved_instance(id)
        .expect("edited instance resolves");
    assert_eq!(resolved.instance().origin(), GridPoint::new(9, 6));
    assert_eq!(resolved.effective_footprint(), GridSize::new(4, 7).unwrap());
}

#[test]
fn four_rotations_restore_rectangular_runtime_buildable_exactly() {
    let buildable_id = BuildableId::new("wide_machine").expect("valid buildable ID");
    let id = EntityId::new(101);
    let mut layout = support::layout_with_buildables(
        GridSize::new(20, 20).expect("positive bounds"),
        &[(
            "wide_machine",
            GridSize::new(7, 4).expect("positive footprint"),
        )],
    );
    layout
        .place(BlockInstance::new(
            id,
            buildable_id,
            GridPoint::new(6, 6),
            Rotation::Zero,
        ))
        .expect("initial placement should fit");
    let original = layout.clone();

    for _ in 0..4 {
        let next_rotation = layout
            .instance(id)
            .expect("instance should remain present")
            .rotation()
            .clockwise();
        layout
            .rotate_instance(id, next_rotation)
            .expect("each in-place rotation should fit");
    }

    assert_eq!(layout, original);
}

#[test]
fn layout_clone_shares_catalog_snapshot_but_not_mutable_instances() {
    let buildable_id = BuildableId::new("wide_machine").expect("valid buildable ID");
    let id = EntityId::new(102);
    let mut original = support::layout_with_buildables(
        GridSize::new(20, 20).expect("positive bounds"),
        &[(
            "wide_machine",
            GridSize::new(7, 4).expect("positive footprint"),
        )],
    );
    original
        .place(BlockInstance::new(
            id,
            buildable_id,
            GridPoint::new(1, 1),
            Rotation::Zero,
        ))
        .expect("initial placement should fit");
    let mut cloned = original.clone();

    assert!(std::ptr::eq(
        original.catalog().buildables(),
        cloned.catalog().buildables()
    ));
    cloned
        .move_instance(id, GridPoint::new(10, 10))
        .expect("clone move should fit");

    assert_eq!(
        original.instance(id).unwrap().origin(),
        GridPoint::new(1, 1)
    );
    assert_eq!(
        cloned.instance(id).unwrap().origin(),
        GridPoint::new(10, 10)
    );
}

#[test]
fn instances_are_enumerated_in_entity_id_order() {
    let high = BlockInstance::new(
        EntityId::new(9),
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let low = BlockInstance::new(
        EntityId::new(2),
        support::buildable_id("refinery_unit"),
        GridPoint::new(3, 0),
        Rotation::Clockwise90,
    );
    let mut layout = main_layout();

    assert_eq!(layout.place(high.clone()), Ok(()));
    assert_eq!(layout.place(low.clone()), Ok(()));

    let instances: Vec<_> = layout.instances().cloned().collect();

    assert_eq!(instances, vec![low, high]);
}

#[test]
fn removing_unknown_instance_returns_none_without_mutation() {
    let existing_id = EntityId::new(10);
    let existing = BlockInstance::new(
        existing_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(existing.clone()), Ok(()));

    assert_eq!(layout.remove_instance(EntityId::new(999)), None);
    assert_eq!(layout.len(), 1);
    assert_eq!(layout.instance(existing_id), Some(&existing));
}

#[test]
fn removing_instance_returns_it_and_preserves_other_instances() {
    let removed_id = EntityId::new(20);
    let removed = BlockInstance::new(
        removed_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let kept_id = EntityId::new(21);
    let kept = BlockInstance::new(
        kept_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(3, 0),
        Rotation::Clockwise180,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(removed.clone()), Ok(()));
    assert_eq!(layout.place(kept.clone()), Ok(()));

    assert_eq!(layout.remove_instance(removed_id), Some(removed));
    assert_eq!(layout.instance(removed_id), None);
    assert_eq!(layout.instance(kept_id), Some(&kept));
    assert_eq!(layout.len(), 1);
    assert!(!layout.is_empty());
}

#[test]
fn moving_unknown_instance_returns_entity_not_found() {
    let id = EntityId::new(100);
    let mut layout = main_layout();

    assert_eq!(
        layout.move_instance(id, GridPoint::new(4, 5)),
        Err(InstanceEditError::EntityNotFound { id })
    );
    assert!(layout.is_empty());
}

#[test]
fn moving_instance_updates_only_origin_and_allows_edge_contact() {
    let blocker_id = EntityId::new(110);
    let blocker = BlockInstance::new(
        blocker_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let moved_id = EntityId::new(111);
    let moved = BlockInstance::new(
        moved_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(6, 0),
        Rotation::Clockwise180,
    );
    let expected = BlockInstance::new(
        moved_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(2, 0),
        Rotation::Clockwise180,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(blocker.clone()), Ok(()));
    assert_eq!(layout.place(moved), Ok(()));

    assert_eq!(layout.move_instance(moved_id, GridPoint::new(2, 0)), Ok(()));
    assert_eq!(layout.instance(moved_id), Some(&expected));
    assert_eq!(layout.instance(blocker_id), Some(&blocker));
    assert_eq!(layout.len(), 2);
}

#[test]
fn moving_out_of_bounds_instance_is_rejected_before_collision() {
    let blocker_id = EntityId::new(120);
    let blocker = BlockInstance::new(
        blocker_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(78, 78),
        Rotation::Zero,
    );
    let moved_id = EntityId::new(121);
    let original = BlockInstance::new(
        moved_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(70, 70),
        Rotation::Clockwise90,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(blocker.clone()), Ok(()));
    assert_eq!(layout.place(original.clone()), Ok(()));

    assert_eq!(
        layout.move_instance(moved_id, GridPoint::new(-1, 70)),
        Err(InstanceEditError::OutOfBounds { id: moved_id })
    );
    assert_eq!(layout.instance(moved_id), Some(&original));

    assert_eq!(
        layout.move_instance(moved_id, GridPoint::new(78, 78)),
        Err(InstanceEditError::OutOfBounds { id: moved_id })
    );
    assert_eq!(layout.instance(moved_id), Some(&original));
    assert_eq!(layout.instance(blocker_id), Some(&blocker));
    assert_eq!(layout.len(), 2);
}

#[test]
fn moving_instance_into_collision_preserves_original_layout() {
    let blocker_id = EntityId::new(130);
    let blocker = BlockInstance::new(
        blocker_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let moved_id = EntityId::new(131);
    let original = BlockInstance::new(
        moved_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(5, 0),
        Rotation::Clockwise270,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(blocker.clone()), Ok(()));
    assert_eq!(layout.place(original.clone()), Ok(()));

    assert_eq!(
        layout.move_instance(moved_id, GridPoint::new(1, 1)),
        Err(InstanceEditError::Collision {
            id: moved_id,
            conflicting_id: blocker_id,
        })
    );
    assert_eq!(layout.instance(moved_id), Some(&original));
    assert_eq!(layout.instance(blocker_id), Some(&blocker));
    assert_eq!(layout.len(), 2);
}

#[test]
fn rotating_unknown_instance_returns_entity_not_found() {
    let id = EntityId::new(200);
    let mut layout = main_layout();

    assert_eq!(
        layout.rotate_instance(id, Rotation::Clockwise90),
        Err(InstanceEditError::EntityNotFound { id })
    );
    assert!(layout.is_empty());
}

#[test]
fn moving_group_ignores_members_old_positions_and_commits_final_layout() {
    let first_id = EntityId::new(300);
    let second_id = EntityId::new(301);
    let first = BlockInstance::new(
        first_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let second = BlockInstance::new(
        second_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(2, 0),
        Rotation::Zero,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(first), Ok(()));
    assert_eq!(layout.place(second), Ok(()));

    assert_eq!(
        layout.move_instances_by(&[first_id, second_id], GridPoint::new(1, 0)),
        Ok(())
    );
    assert_eq!(
        layout.instance(first_id).map(|instance| instance.origin()),
        Some(GridPoint::new(1, 0))
    );
    assert_eq!(
        layout.instance(second_id).map(|instance| instance.origin()),
        Some(GridPoint::new(3, 0))
    );
}

#[test]
fn rejected_group_move_rolls_back_every_member() {
    let first_id = EntityId::new(310);
    let second_id = EntityId::new(311);
    let blocker_id = EntityId::new(312);
    let first = BlockInstance::new(
        first_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let second = BlockInstance::new(
        second_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(2, 0),
        Rotation::Zero,
    );
    let blocker = BlockInstance::new(
        blocker_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(5, 0),
        Rotation::Zero,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(first), Ok(()));
    assert_eq!(layout.place(second), Ok(()));
    assert_eq!(layout.place(blocker), Ok(()));
    let before = layout.clone();

    assert_eq!(
        layout.move_instances_by(&[first_id, second_id], GridPoint::new(2, 0)),
        Err(InstanceEditError::Collision {
            id: second_id,
            conflicting_id: blocker_id,
        })
    );
    assert_eq!(layout, before);
}

#[test]
fn selection_rotation_pivot_uses_physical_bounds_and_snaps_toward_top_left() {
    let pole_id = EntityId::new(330);
    let refinery_id = EntityId::new(331);
    let mut layout = main_layout();
    assert_eq!(
        layout.place(BlockInstance::new(
            pole_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(10, 10),
            Rotation::Zero,
        )),
        Ok(())
    );
    assert_eq!(
        layout.place(BlockInstance::new(
            refinery_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(14, 10),
            Rotation::Zero,
        )),
        Ok(())
    );

    assert_eq!(
        layout.selection_rotation_pivot(&[refinery_id, pole_id]),
        Ok(Some(GridPoint::new(13, 11)))
    );
}

#[test]
fn selection_rotation_pivot_requires_multiple_existing_instances() {
    let existing_id = EntityId::new(335);
    let missing_id = EntityId::new(336);
    let mut layout = main_layout();

    assert_eq!(layout.selection_rotation_pivot(&[]), Ok(None));
    assert_eq!(
        layout.place(BlockInstance::new(
            existing_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(10, 10),
            Rotation::Zero,
        )),
        Ok(())
    );
    assert_eq!(
        layout.selection_rotation_pivot(&[existing_id, existing_id]),
        Ok(None)
    );
    assert_eq!(
        layout.selection_rotation_pivot(&[existing_id, missing_id]),
        Err(InstanceEditError::EntityNotFound { id: missing_id })
    );
}

#[test]
fn rotating_group_about_center_moves_origins_and_orientations() {
    let first_id = EntityId::new(340);
    let second_id = EntityId::new(341);
    let mut layout = main_layout();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
    ] {
        assert_eq!(
            layout.place(BlockInstance::new(
                id,
                support::buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    let pivot = GridPoint::new(13, 11);

    assert_eq!(
        layout.rotate_instances_clockwise_about(&[first_id, second_id], pivot),
        Ok(())
    );
    assert_eq!(
        layout.instance(first_id).cloned(),
        Some(BlockInstance::new(
            first_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(12, 8),
            Rotation::Clockwise90,
        ))
    );
    assert_eq!(
        layout.instance(second_id).cloned(),
        Some(BlockInstance::new(
            second_id,
            support::buildable_id("xiranite_power_pole"),
            GridPoint::new(12, 12),
            Rotation::Clockwise90,
        ))
    );
}

#[test]
fn four_group_rotations_about_same_pivot_restore_original_layout() {
    let first_id = EntityId::new(350);
    let second_id = EntityId::new(351);
    let ids = [first_id, second_id];
    let mut layout = main_layout();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
    ] {
        assert_eq!(
            layout.place(BlockInstance::new(
                id,
                support::buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    let original = layout.clone();
    let pivot = layout
        .selection_rotation_pivot(&ids)
        .expect("selected instances exist")
        .expect("multiple instances have a group pivot");

    for _ in 0..4 {
        assert_eq!(layout.rotate_instances_clockwise_about(&ids, pivot), Ok(()));
    }

    assert_eq!(layout, original);
}

#[test]
fn group_rotation_into_external_block_rolls_back_every_member() {
    let first_id = EntityId::new(360);
    let second_id = EntityId::new(361);
    let blocker_id = EntityId::new(362);
    let mut layout = main_layout();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
        (blocker_id, GridPoint::new(12, 8)),
    ] {
        assert_eq!(
            layout.place(BlockInstance::new(
                id,
                support::buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    let before = layout.clone();

    assert_eq!(
        layout.rotate_instances_clockwise_about(&[first_id, second_id], GridPoint::new(13, 11),),
        Err(InstanceEditError::Collision {
            id: first_id,
            conflicting_id: blocker_id,
        })
    );
    assert_eq!(layout, before);
}

#[test]
fn group_rotation_out_of_bounds_rolls_back_every_member() {
    let first_id = EntityId::new(370);
    let second_id = EntityId::new(371);
    let mut layout = main_layout();
    for (id, origin) in [
        (first_id, GridPoint::new(0, 0)),
        (second_id, GridPoint::new(4, 0)),
    ] {
        assert_eq!(
            layout.place(BlockInstance::new(
                id,
                support::buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    let before = layout.clone();

    assert_eq!(
        layout.rotate_instances_clockwise_about(&[first_id, second_id], GridPoint::new(3, 1),),
        Err(InstanceEditError::OutOfBounds { id: first_id })
    );
    assert_eq!(layout, before);
}

#[test]
fn group_rotation_with_missing_id_preserves_complete_layout() {
    let existing_id = EntityId::new(380);
    let missing_id = EntityId::new(381);
    let existing = BlockInstance::new(
        existing_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(10, 10),
        Rotation::Zero,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(existing.clone()), Ok(()));

    assert_eq!(
        layout
            .rotate_instances_clockwise_about(&[existing_id, missing_id], GridPoint::new(11, 11),),
        Err(InstanceEditError::EntityNotFound { id: missing_id })
    );
    assert_eq!(layout.instance(existing_id), Some(&existing));
    assert_eq!(layout.len(), 1);
}

#[test]
fn group_rotation_coordinate_overflow_is_out_of_bounds_and_rolls_back() {
    let first_id = EntityId::new(390);
    let second_id = EntityId::new(391);
    let mut layout = main_layout();
    for (id, origin) in [
        (first_id, GridPoint::new(10, 10)),
        (second_id, GridPoint::new(14, 10)),
    ] {
        assert_eq!(
            layout.place(BlockInstance::new(
                id,
                support::buildable_id("xiranite_power_pole"),
                origin,
                Rotation::Zero,
            )),
            Ok(())
        );
    }
    let before = layout.clone();

    assert_eq!(
        layout.rotate_instances_clockwise_about(
            &[first_id, second_id],
            GridPoint::new(i32::MAX, i32::MAX),
        ),
        Err(InstanceEditError::OutOfBounds { id: first_id })
    );
    assert_eq!(layout, before);
}

#[test]
fn rotating_instance_updates_only_rotation() {
    let id = EntityId::new(210);
    let original = BlockInstance::new(
        id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(10, 10),
        Rotation::Zero,
    );
    let expected = BlockInstance::new(
        id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(10, 10),
        Rotation::Clockwise90,
    );
    let mut layout = main_layout();
    assert_eq!(layout.place(original), Ok(()));

    assert_eq!(layout.rotate_instance(id, Rotation::Clockwise90), Ok(()));
    assert_eq!(layout.instance(id), Some(&expected));
    assert_eq!(layout.len(), 1);
}
