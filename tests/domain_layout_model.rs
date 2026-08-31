use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, Catalog, CatalogId, CatalogMetadata, RegionDefinition, RegionId,
};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{
    BlockInstance, EntityId, FactoryLayout, LayoutCreationError, PlacementError,
};
use semver::Version;

mod support;

fn layout_with_bounds(bounds: GridSize) -> FactoryLayout {
    let catalog_id = CatalogId::new("layout_test_catalog").expect("valid catalog ID");
    let region_id = RegionId::new("layout_test_region").expect("valid region ID");
    let base_id = BaseId::new("layout_test_base").expect("valid base ID");
    let catalog = Catalog::new(
        CatalogMetadata::new(catalog_id, Version::new(1, 0, 0), "Layout Test Catalog"),
        base_id.clone(),
        vec![RegionDefinition::new(
            region_id.clone(),
            "Layout Test Region",
        )],
        vec![BaseDefinition::new(
            base_id.clone(),
            "Layout Test Base",
            region_id,
            bounds,
        )],
        vec![],
        vec![],
    )
    .expect("synthetic layout catalog should be valid");

    FactoryLayout::new(catalog, base_id).expect("synthetic base should exist")
}

#[test]
fn entity_id_preserves_its_numeric_value() {
    let id = EntityId::new(42);

    assert_eq!(id.value(), 42);
}

#[test]
fn block_instance_preserves_structural_data() {
    let id = EntityId::new(7);
    let template = support::buildable_id("refinery_unit");
    let origin = GridPoint::new(-2, 5);
    let rotation = Rotation::Clockwise90;

    let instance = BlockInstance::new(id, template.clone(), origin, rotation);

    assert_eq!(instance.id(), id);
    assert_eq!(instance.buildable_id(), &template);
    assert_eq!(instance.origin(), origin);
    assert_eq!(instance.rotation(), rotation);
}

#[test]
fn factory_layout_derives_bounds_from_runtime_base_definition() {
    let expected_bounds = GridSize::new(7, 5).expect("test base dimensions are positive");

    let layout = layout_with_bounds(expected_bounds);

    assert_eq!(layout.base_id().as_str(), "layout_test_base");
    assert_eq!(layout.base_definition().display_name(), "Layout Test Base");
    assert_eq!(layout.bounds(), expected_bounds);
}

#[test]
fn factory_layout_rejects_base_missing_from_runtime_catalog() {
    let layout = layout_with_bounds(GridSize::new(7, 5).expect("positive test bounds"));
    let catalog = layout.catalog().clone();
    let missing_base_id = BaseId::new("missing_base").expect("valid missing base ID");

    assert_eq!(
        FactoryLayout::new(catalog, missing_base_id.clone()),
        Err(LayoutCreationError::BaseNotFound {
            base_id: missing_base_id,
        })
    );
}

#[test]
fn layout_creation_error_is_an_actionable_standard_error() {
    fn assert_standard_error<T: std::error::Error>() {}

    assert_standard_error::<LayoutCreationError>();

    let layout = layout_with_bounds(GridSize::new(7, 5).expect("positive test bounds"));
    let catalog = layout.catalog().clone();
    let missing_base_id = BaseId::new("missing_base").expect("valid missing base ID");
    let error = FactoryLayout::new(catalog, missing_base_id).unwrap_err();

    assert_eq!(
        error.to_string(),
        "base 'missing_base' does not exist in the catalog"
    );
}

#[test]
fn placement_uses_runtime_rectangular_base_bounds() {
    let mut layout = support::layout_with_buildables(
        GridSize::new(7, 5).expect("positive test bounds"),
        &[(
            "refinery_unit",
            GridSize::new(3, 3).expect("positive footprint"),
        )],
    );
    let exact_fit_id = EntityId::new(40);
    let out_of_bounds_id = EntityId::new(41);

    assert_eq!(
        layout.place(BlockInstance::new(
            exact_fit_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(4, 2),
            Rotation::Zero,
        )),
        Ok(())
    );
    assert_eq!(
        layout.place(BlockInstance::new(
            out_of_bounds_id,
            support::buildable_id("refinery_unit"),
            GridPoint::new(5, 2),
            Rotation::Zero,
        )),
        Err(PlacementError::OutOfBounds {
            id: out_of_bounds_id,
        })
    );
    assert_eq!(layout.len(), 1);
}

#[test]
fn instance_at_finds_occupied_tiles_and_respects_half_open_edges() {
    let pole_id = EntityId::new(8);
    let pole = BlockInstance::new(
        pole_id,
        support::buildable_id("xiranite_power_pole"),
        GridPoint::new(0, 0),
        Rotation::Zero,
    );
    let refinery_id = EntityId::new(9);
    let refinery = BlockInstance::new(
        refinery_id,
        support::buildable_id("refinery_unit"),
        GridPoint::new(2, 0),
        Rotation::Zero,
    );
    let mut layout = support::layout_with_buildables(
        GridSize::new(80, 80).expect("positive test bounds"),
        &[
            (
                "xiranite_power_pole",
                GridSize::new(2, 2).expect("positive footprint"),
            ),
            (
                "refinery_unit",
                GridSize::new(3, 3).expect("positive footprint"),
            ),
        ],
    );
    assert_eq!(layout.place(pole.clone()), Ok(()));
    assert_eq!(layout.place(refinery.clone()), Ok(()));

    assert_eq!(layout.instance_at(GridPoint::new(0, 0)), Some(&pole));
    assert_eq!(layout.instance_at(GridPoint::new(1, 1)), Some(&pole));
    assert_eq!(layout.instance_at(GridPoint::new(2, 0)), Some(&refinery));
    assert_eq!(layout.instance_at(GridPoint::new(4, 2)), Some(&refinery));
    assert_eq!(layout.instance_at(GridPoint::new(5, 0)), None);
    assert_eq!(layout.instance_at(GridPoint::new(0, 2)), None);
    assert_eq!(layout.instance_at(GridPoint::new(-1, 0)), None);
    assert_eq!(layout.instance_at(GridPoint::new(80, 0)), None);
}
