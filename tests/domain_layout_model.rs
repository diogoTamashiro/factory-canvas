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

#[test]
fn instance_at_finds_occupied_tiles_and_respects_half_open_edges() {
    let pole_id = EntityId::new(8);
    let pole = BlockInstance::new(
        pole_id,
        BlockTemplate::XiranitePowerPole,
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let refinery_id = EntityId::new(9);
    let refinery = BlockInstance::new(
        refinery_id,
        BlockTemplate::RefineryUnit,
        GridPoint::new(2, 0),
        Rotation::Zero,
    );
    let mut layout = FactoryLayout::new(BaseTemplate::MainCurrent);
    assert_eq!(layout.place(pole), Ok(()));
    assert_eq!(layout.place(refinery), Ok(()));

    assert_eq!(layout.instance_at(GridPoint::new(0, 0)), Some(&pole));
    assert_eq!(layout.instance_at(GridPoint::new(1, 1)), Some(&pole));
    assert_eq!(layout.instance_at(GridPoint::new(2, 0)), Some(&refinery));
    assert_eq!(layout.instance_at(GridPoint::new(4, 2)), Some(&refinery));
    assert_eq!(layout.instance_at(GridPoint::new(5, 0)), None);
    assert_eq!(layout.instance_at(GridPoint::new(0, 2)), None);
    assert_eq!(layout.instance_at(GridPoint::new(-1, 0)), None);
    assert_eq!(layout.instance_at(GridPoint::new(80, 0)), None);
}
