use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::geometry::GridSize;

#[test]
fn public_catalog_preserves_compatibility_buildable_contract() {
    let catalog = load_embedded_public_catalog().expect("public catalog must load");
    let actual: Vec<_> = catalog
        .buildables()
        .iter()
        .map(|definition| {
            (
                definition.id().as_str(),
                definition.display_name(),
                definition.category_id().as_str(),
                definition.symbol(),
                definition.footprint(),
            )
        })
        .collect();

    assert_eq!(
        actual,
        vec![
            (
                "xiranite_power_pole",
                "Xiranite Power Pole",
                "energy",
                "XPP",
                GridSize::new(2, 2).unwrap(),
            ),
            (
                "refinery_unit",
                "Refinery Unit",
                "production_i",
                "RU",
                GridSize::new(3, 3).unwrap(),
            ),
            (
                "crushing_unit",
                "Crushing Unit",
                "production_i",
                "CU",
                GridSize::new(3, 3).unwrap(),
            ),
        ]
    );
}
