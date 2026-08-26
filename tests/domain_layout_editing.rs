use factory_canvas::domain::base::BaseTemplate;
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout, InstanceEditError};

#[test]
fn instances_are_enumerated_in_entity_id_order() {
    let high = BlockInstance::new(
        EntityId::new(9),
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let low = BlockInstance::new(
        EntityId::new(2),
        BlockTemplate::RefineryUnit,
        GridPoint::new(3, 0),
        Rotation::Clockwise90,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);

    assert_eq!(layout.place(high), Ok(()));
    assert_eq!(layout.place(low), Ok(()));

    let instances: Vec<_> = layout.instances().copied().collect();

    assert_eq!(instances, vec![low, high]);
}

#[test]
fn removing_unknown_instance_returns_none_without_mutation() {
    let existing_id = EntityId::new(10);
    let existing = BlockInstance::new(
        existing_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(existing), Ok(()));

    assert_eq!(layout.remove_instance(EntityId::new(999)), None);
    assert_eq!(layout.len(), 1);
    assert_eq!(layout.instance(existing_id), Some(&existing));
}

#[test]
fn removing_instance_returns_it_and_preserves_other_instances() {
    let removed_id = EntityId::new(20);
    let removed = BlockInstance::new(
        removed_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let kept_id = EntityId::new(21);
    let kept = BlockInstance::new(
        kept_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(3, 0),
        Rotation::Clockwise180,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(removed), Ok(()));
    assert_eq!(layout.place(kept), Ok(()));

    assert_eq!(layout.remove_instance(removed_id), Some(removed));
    assert_eq!(layout.instance(removed_id), None);
    assert_eq!(layout.instance(kept_id), Some(&kept));
    assert_eq!(layout.len(), 1);
    assert!(!layout.is_empty());
}

#[test]
fn moving_unknown_instance_returns_entity_not_found() {
    let id = EntityId::new(100);
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);

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
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let moved_id = EntityId::new(111);
    let moved = BlockInstance::new(
        moved_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(6, 0),
        Rotation::Clockwise180,
    );
    let expected = BlockInstance::new(
        moved_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(2, 0),
        Rotation::Clockwise180,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(blocker), Ok(()));
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
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(78, 78),
        Rotation::Zero,
    );
    let moved_id = EntityId::new(121);
    let original = BlockInstance::new(
        moved_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(70, 70),
        Rotation::Clockwise90,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(blocker), Ok(()));
    assert_eq!(layout.place(original), Ok(()));

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
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let moved_id = EntityId::new(131);
    let original = BlockInstance::new(
        moved_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(5, 0),
        Rotation::Clockwise270,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(blocker), Ok(()));
    assert_eq!(layout.place(original), Ok(()));

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
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);

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
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let second = BlockInstance::new(
        second_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(2, 0),
        Rotation::Zero,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
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
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let second = BlockInstance::new(
        second_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(2, 0),
        Rotation::Zero,
    );
    let blocker = BlockInstance::new(
        blocker_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(5, 0),
        Rotation::Zero,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
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
fn rotating_group_updates_every_member_atomically() {
    let first_id = EntityId::new(320);
    let second_id = EntityId::new(321);
    let first = BlockInstance::new(
        first_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let second = BlockInstance::new(
        second_id,
        BlockTemplate::CrushingUnit,
        GridPoint::new(4, 0),
        Rotation::Clockwise90,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(first), Ok(()));
    assert_eq!(layout.place(second), Ok(()));

    assert_eq!(
        layout.rotate_instances_clockwise(&[first_id, second_id]),
        Ok(())
    );
    assert_eq!(
        layout
            .instance(first_id)
            .map(|instance| instance.rotation()),
        Some(Rotation::Clockwise90)
    );
    assert_eq!(
        layout
            .instance(second_id)
            .map(|instance| instance.rotation()),
        Some(Rotation::Clockwise180)
    );
}

#[test]
fn rotating_instance_updates_only_rotation() {
    let id = EntityId::new(210);
    let original = BlockInstance::new(
        id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(10, 10),
        Rotation::Zero,
    );
    let expected = BlockInstance::new(
        id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(10, 10),
        Rotation::Clockwise90,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(original), Ok(()));

    assert_eq!(layout.rotate_instance(id, Rotation::Clockwise90), Ok(()));
    assert_eq!(layout.instance(id), Some(&expected));
    assert_eq!(layout.len(), 1);
}
