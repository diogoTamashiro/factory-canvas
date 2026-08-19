use factory_canvas::domain::base::{BaseTemplate, SecondaryLevel};
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{BlockInstance, EntityId, FactoryLayout};

#[test]
fn entity_id_preserves_its_numeric_value() {
    let id = EntityId::new(42);

    assert_eq!(id.value(), 42);
}

#[test]
fn block_instance_preserves_structural_data() {
    let id = EntityId::new(7);
    let template = BlockTemplate::RefineryUnit;
    let origin = GridPoint::new(-2, 5);
    let rotation = Rotation::Clockwise90;

    let instance = BlockInstance::new(id, template, origin, rotation);

    assert_eq!(instance.id(), id);
    assert_eq!(instance.template(), template);
    assert_eq!(instance.origin(), origin);
    assert_eq!(instance.rotation(), rotation);
}

#[test]
fn factory_layout_derives_bounds_from_selected_base_template() {
    let template = BaseTemplate::Secondary(SecondaryLevel::AreaExpansionI);
    let expected_bounds = GridSize::new(40, 40).expect("confirmed base dimensions are positive");

    let layout = FactoryLayout::new(template);

    assert_eq!(layout.base_template(), template);
    assert_eq!(layout.bounds(), expected_bounds);
}
