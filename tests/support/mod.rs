use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CategoryId, RegionDefinition, RegionId,
};
use factory_canvas::domain::geometry::GridSize;
use factory_canvas::domain::layout::FactoryLayout;
use semver::Version;

pub fn layout_with_buildables(
    base_bounds: GridSize,
    buildables: &[(&str, GridSize)],
) -> FactoryLayout {
    let region_id = RegionId::new("test_region").expect("valid test region ID");
    let base_id = BaseId::new("test_base").expect("valid test base ID");
    let category_id = CategoryId::new("test_category").expect("valid test category ID");
    let buildables = buildables
        .iter()
        .map(|(id, footprint)| {
            BuildableDefinition::new(
                BuildableId::new(*id).expect("valid test buildable ID"),
                *id,
                category_id.clone(),
                "T",
                *footprint,
                Vec::new(),
            )
        })
        .collect();
    let catalog = Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("test_catalog").expect("valid test catalog ID"),
            Version::new(1, 0, 0),
            "Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(region_id.clone(), "Test Region")],
        vec![BaseDefinition::new(
            base_id.clone(),
            "Test Base",
            region_id,
            base_bounds,
        )],
        buildables,
        Vec::new(),
    )
    .expect("valid test catalog");

    FactoryLayout::new(catalog, base_id).expect("test base exists")
}
