use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::catalog::BlockTemplate;
use factory_canvas::domain::geometry::GridSize;

#[test]
fn temporary_templates_resolve_exact_public_catalog_definitions_by_id() {
    let catalog = load_embedded_public_catalog().expect("public catalog must load");
    let cases = [
        (
            BlockTemplate::XiranitePowerPole,
            "xiranite_power_pole",
            "Xiranite Power Pole",
            "energy",
            GridSize::new(2, 2).unwrap(),
        ),
        (
            BlockTemplate::RefineryUnit,
            "refinery_unit",
            "Refinery Unit",
            "production_i",
            GridSize::new(3, 3).unwrap(),
        ),
        (
            BlockTemplate::CrushingUnit,
            "crushing_unit",
            "Crushing Unit",
            "production_i",
            GridSize::new(3, 3).unwrap(),
        ),
    ];

    assert_eq!(BlockTemplate::ALL.len(), cases.len());
    for (template, expected_id, expected_name, expected_category, expected_footprint) in cases {
        let buildable_id = template.buildable_id();
        let definition = catalog
            .buildable(&buildable_id)
            .expect("adapter ID exists in public catalog");

        assert_eq!(buildable_id.as_str(), expected_id);
        assert_eq!(definition.display_name(), expected_name);
        assert_eq!(definition.category_id().as_str(), expected_category);
        assert_eq!(definition.footprint(), expected_footprint);
        assert_eq!(
            BlockTemplate::from_buildable_id(&buildable_id),
            Some(template)
        );
    }
}
