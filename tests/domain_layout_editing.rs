use factory_canvas::domain::base::BaseTemplate;
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout};

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
