use factory_canvas::domain::catalog::{BlockCategory, BlockTemplate};
use factory_canvas::domain::geometry::GridSize;

#[test]
fn xiranite_power_pole_definition_matches_confirmed_data() {
    let expected_footprint =
        GridSize::new(2, 2).expect("confirmed footprint dimensions are positive");
    let template = BlockTemplate::ALL[0];
    let definition = template.definition();

    assert_eq!(template, BlockTemplate::XiranitePowerPole);
    assert_eq!(definition.id(), "xiranite_power_pole");
    assert_eq!(definition.display_name(), "Xiranite Power Pole");
    assert_eq!(definition.category(), BlockCategory::Energy);
    assert_eq!(definition.footprint(), expected_footprint);
}

#[test]
fn refinery_definition_matches_confirmed_data() {
    let expected_footprint =
        GridSize::new(3, 3).expect("confirmed footprint dimensions are positive");
    let template = BlockTemplate::ALL[1];
    let definition = template.definition();

    assert_eq!(template, BlockTemplate::RefineryUnit);
    assert_eq!(definition.id(), "refinery_unit");
    assert_eq!(definition.display_name(), "Refinery Unit");
    assert_eq!(definition.category(), BlockCategory::ProductionI);
    assert_eq!(definition.footprint(), expected_footprint);
}

#[test]
fn initial_catalog_ends_with_confirmed_crushing_unit() {
    let expected_footprint =
        GridSize::new(3, 3).expect("confirmed footprint dimensions are positive");

    assert_eq!(BlockTemplate::ALL.len(), 3);

    let template = BlockTemplate::ALL[2];
    let definition = template.definition();

    assert_eq!(template, BlockTemplate::CrushingUnit);
    assert_eq!(definition.id(), "crushing_unit");
    assert_eq!(definition.display_name(), "Crushing Unit");
    assert_eq!(definition.category(), BlockCategory::ProductionI);
    assert_eq!(definition.footprint(), expected_footprint);
}
