use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CategoryId, ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::domain::layout::{
    BlockInstance, EntityId, FactoryLayout, InstanceEditError, PlacementError,
    ProductionTargetError,
};
use semver::Version;

fn product_id(value: &str) -> ProductId {
    ProductId::new(value).expect("test product ID must be valid")
}

fn layout_with_catalog(
    catalog_id: &str,
    products: &[&str],
    production_targets: &[&str],
) -> FactoryLayout {
    let region_id = RegionId::new("region").unwrap();
    let base_id = BaseId::new("base").unwrap();
    let catalog = Catalog::new(
        CatalogMetadata::new(
            CatalogId::new(catalog_id).unwrap(),
            Version::parse("1.0.0").unwrap(),
            "Production Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(region_id.clone(), "Region")],
        vec![BaseDefinition::new(
            base_id.clone(),
            "Base",
            region_id,
            GridSize::new(20, 20).unwrap(),
        )],
        vec![BuildableDefinition::new(
            BuildableId::new("machine").unwrap(),
            "Machine",
            CategoryId::new("production").unwrap(),
            "M",
            GridSize::new(2, 3).unwrap(),
            production_targets
                .iter()
                .map(|value| product_id(value))
                .collect(),
        )],
        products
            .iter()
            .map(|value| ProductDefinition::new(product_id(value), *value))
            .collect(),
    )
    .unwrap();
    FactoryLayout::new(catalog, base_id).unwrap()
}

fn layout() -> FactoryLayout {
    layout_with_catalog(
        "production_catalog",
        &["allowed_product", "other_product"],
        &["allowed_product"],
    )
}

fn placed_layout() -> (FactoryLayout, EntityId) {
    let mut layout = layout();
    let id = EntityId::new(1);
    layout
        .place(BlockInstance::new(
            id,
            BuildableId::new("machine").unwrap(),
            GridPoint::new(2, 3),
            Rotation::Zero,
        ))
        .unwrap();
    (layout, id)
}

fn placed_configured_group() -> (FactoryLayout, [EntityId; 2], ProductId) {
    let mut layout = layout();
    let ids = [EntityId::new(1), EntityId::new(2)];
    let target = product_id("allowed_product");
    for (id, origin) in [
        (ids[0], GridPoint::new(2, 3)),
        (ids[1], GridPoint::new(8, 3)),
    ] {
        layout
            .place(BlockInstance::new(
                id,
                BuildableId::new("machine").unwrap(),
                origin,
                Rotation::Zero,
            ))
            .unwrap();
        layout
            .set_production_target(id, Some(target.clone()))
            .unwrap();
    }
    (layout, ids, target)
}

#[test]
fn production_target_is_validated_and_preserved_by_edits() {
    let (mut layout, id) = placed_layout();
    let original = layout.instance(id).unwrap().clone();
    let allowed = product_id("allowed_product");

    assert_eq!(
        layout.set_production_target(id, Some(allowed.clone())),
        Ok(())
    );
    let configured = layout.instance(id).unwrap();
    assert_eq!(configured.production_target(), Some(&allowed));
    assert_eq!(configured.id(), original.id());
    assert_eq!(configured.buildable_id(), original.buildable_id());
    assert_eq!(configured.origin(), original.origin());
    assert_eq!(configured.rotation(), original.rotation());

    let configured_snapshot = layout.clone();
    assert_eq!(
        layout.set_production_target(EntityId::new(999), Some(allowed.clone())),
        Err(ProductionTargetError::EntityNotFound {
            id: EntityId::new(999)
        })
    );
    assert_eq!(layout, configured_snapshot);
    assert!(matches!(
        layout.set_production_target(id, Some(product_id("missing_product"))),
        Err(ProductionTargetError::ProductNotFound { .. })
    ));
    assert_eq!(layout, configured_snapshot);
    assert!(matches!(
        layout.set_production_target(id, Some(product_id("other_product"))),
        Err(ProductionTargetError::UnsupportedProduct { .. })
    ));
    assert_eq!(layout, configured_snapshot);

    layout.move_instance(id, GridPoint::new(4, 5)).unwrap();
    layout.rotate_instance(id, Rotation::Clockwise90).unwrap();
    assert_eq!(
        layout.instance(id).unwrap().production_target(),
        Some(&allowed)
    );

    assert_eq!(layout.set_production_target(id, None), Ok(()));
    assert_eq!(layout.instance(id).unwrap().production_target(), None);
}

#[test]
fn missing_entity_precedes_target_validation_and_clear() {
    let (mut layout, _) = placed_layout();
    let missing_id = EntityId::new(999);
    let before = layout.clone();

    assert_eq!(
        layout.set_production_target(missing_id, Some(product_id("missing_product"))),
        Err(ProductionTargetError::EntityNotFound { id: missing_id })
    );
    assert_eq!(layout, before);
    assert_eq!(
        layout.set_production_target(missing_id, None),
        Err(ProductionTargetError::EntityNotFound { id: missing_id })
    );
    assert_eq!(layout, before);
}

#[test]
fn group_move_preserves_configured_targets() {
    let (mut layout, ids, target) = placed_configured_group();

    layout
        .move_instances_by(&ids, GridPoint::new(1, 2))
        .unwrap();

    assert_eq!(
        layout.instance(ids[0]).unwrap().origin(),
        GridPoint::new(3, 5)
    );
    assert_eq!(
        layout.instance(ids[1]).unwrap().origin(),
        GridPoint::new(9, 5)
    );
    for id in ids {
        assert_eq!(
            layout.instance(id).unwrap().production_target(),
            Some(&target)
        );
    }
}

#[test]
fn group_rotation_preserves_configured_targets() {
    let (mut layout, ids, target) = placed_configured_group();

    layout
        .rotate_instances_clockwise_about(&ids, GridPoint::new(7, 7))
        .unwrap();

    assert_eq!(
        layout.instance(ids[0]).unwrap().origin(),
        GridPoint::new(8, 2)
    );
    assert_eq!(
        layout.instance(ids[1]).unwrap().origin(),
        GridPoint::new(8, 8)
    );
    for id in ids {
        let instance = layout.instance(id).unwrap();
        assert_eq!(instance.rotation(), Rotation::Clockwise90);
        assert_eq!(instance.production_target(), Some(&target));
    }
}

#[test]
fn rejected_single_edit_preserves_configured_target() {
    let (mut layout, id) = placed_layout();
    layout
        .set_production_target(id, Some(product_id("allowed_product")))
        .unwrap();
    let before = layout.clone();

    assert_eq!(
        layout.move_instance(id, GridPoint::new(-1, 3)),
        Err(InstanceEditError::OutOfBounds { id })
    );
    assert_eq!(layout, before);
}

#[test]
fn rejected_group_edit_preserves_configured_targets() {
    let (mut layout, ids, _) = placed_configured_group();
    let before = layout.clone();

    assert_eq!(
        layout.move_instances_by(&ids, GridPoint::new(-3, 0)),
        Err(InstanceEditError::OutOfBounds { id: ids[0] })
    );
    assert_eq!(layout, before);
}

#[test]
fn placement_accepts_configured_target_supported_by_destination_catalog() {
    let (mut source, id) = placed_layout();
    let target = product_id("allowed_product");
    source
        .set_production_target(id, Some(target.clone()))
        .unwrap();
    let configured = source.remove_instance(id).unwrap();
    let mut destination = layout_with_catalog(
        "destination_catalog",
        &["allowed_product"],
        &["allowed_product"],
    );

    assert_eq!(destination.place(configured.clone()), Ok(()));
    assert_eq!(destination.instance(id), Some(&configured));
    assert_eq!(
        destination.instance(id).unwrap().production_target(),
        Some(&target)
    );
}

#[test]
fn placement_rejects_configured_target_missing_from_destination_catalog() {
    let (mut source, id) = placed_layout();
    let target = product_id("allowed_product");
    source
        .set_production_target(id, Some(target.clone()))
        .unwrap();
    let configured = source.remove_instance(id).unwrap();
    let mut destination = layout_with_catalog("destination_catalog", &[], &[]);
    let before = destination.clone();

    assert_eq!(
        destination.place(configured),
        Err(PlacementError::ProductNotFound {
            id,
            product_id: target,
        })
    );
    assert_eq!(destination, before);
}

#[test]
fn placement_rejects_configured_target_unsupported_by_destination_buildable() {
    let (mut source, id) = placed_layout();
    let buildable_id = BuildableId::new("machine").unwrap();
    let target = product_id("allowed_product");
    source
        .set_production_target(id, Some(target.clone()))
        .unwrap();
    let configured = source.remove_instance(id).unwrap();
    let mut destination = layout_with_catalog("destination_catalog", &["allowed_product"], &[]);
    let before = destination.clone();

    assert_eq!(
        destination.place(configured),
        Err(PlacementError::UnsupportedProduct {
            id,
            buildable_id,
            product_id: target,
        })
    );
    assert_eq!(destination, before);
}
