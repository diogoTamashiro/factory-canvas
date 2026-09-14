use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CatalogValidationError, CategoryId, ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use factory_canvas::domain::geometry::GridSize;
use semver::Version;

#[test]
fn valid_catalog_id_preserves_its_value() {
    let id = CatalogId::new("factory_canvas_public")
        .expect("ASCII snake_case catalog IDs should be valid");

    assert_eq!(id.as_str(), "factory_canvas_public");
}

#[test]
fn catalog_id_rejects_values_outside_ascii_snake_case() {
    for invalid in [
        "",
        "1catalog",
        "factory-canvas",
        "factory canvas",
        "factory__canvas",
        "factory_canvas_",
        "_factory_canvas",
        "Factory_canvas",
        "fábrica_canvas",
    ] {
        assert!(
            CatalogId::new(invalid).is_err(),
            "{invalid:?} must not be accepted as a catalog ID"
        );
    }
}

#[test]
fn catalog_metadata_exposes_typed_identity_version_and_name() {
    let catalog_id = CatalogId::new("factory_canvas_public").expect("valid catalog ID");
    let data_version = Version::parse("0.1.0").expect("valid semantic version");

    let metadata = CatalogMetadata::new(
        catalog_id.clone(),
        data_version.clone(),
        "Factory Canvas - Public Catalog",
    );

    assert_eq!(metadata.catalog_id(), &catalog_id);
    assert_eq!(metadata.data_version(), &data_version);
    assert_eq!(metadata.display_name(), "Factory Canvas - Public Catalog");
}

struct CatalogInput {
    metadata: CatalogMetadata,
    default_base_id: BaseId,
    regions: Vec<RegionDefinition>,
    bases: Vec<BaseDefinition>,
    buildables: Vec<BuildableDefinition>,
    products: Vec<ProductDefinition>,
}

impl CatalogInput {
    fn valid() -> Self {
        let wuling = RegionId::new("wuling").expect("valid region ID");
        let valley = RegionId::new("valley").expect("valid region ID");
        let iron = ProductId::new("iron").expect("valid product ID");

        Self {
            metadata: CatalogMetadata::new(
                CatalogId::new("synthetic_catalog").expect("valid catalog ID"),
                Version::parse("1.2.3").expect("valid semantic version"),
                "Synthetic Catalog",
            ),
            default_base_id: BaseId::new("main_base").expect("valid base ID"),
            regions: vec![
                RegionDefinition::new(wuling.clone(), "Wuling"),
                RegionDefinition::new(valley.clone(), "Valley"),
            ],
            bases: vec![
                BaseDefinition::new(
                    BaseId::new("main_base").expect("valid base ID"),
                    "Main Base",
                    wuling,
                    GridSize::new(10, 8).expect("positive bounds"),
                ),
                BaseDefinition::new(
                    BaseId::new("auxiliary_base").expect("valid base ID"),
                    "Auxiliary Base",
                    valley,
                    GridSize::new(6, 4).expect("positive bounds"),
                ),
            ],
            buildables: vec![
                BuildableDefinition::new(
                    BuildableId::new("smelter").expect("valid buildable ID"),
                    "Smelter",
                    CategoryId::new("production").expect("valid category ID"),
                    "SM",
                    GridSize::new(2, 3).expect("positive footprint"),
                    vec![iron.clone()],
                ),
                BuildableDefinition::new(
                    BuildableId::new("battery").expect("valid buildable ID"),
                    "Battery",
                    CategoryId::new("energy").expect("valid category ID"),
                    "BT",
                    GridSize::new(1, 1).expect("positive footprint"),
                    vec![],
                ),
            ],
            products: vec![
                ProductDefinition::new(iron, "Iron"),
                ProductDefinition::new(
                    ProductId::new("copper").expect("valid product ID"),
                    "Copper",
                ),
            ],
        }
    }

    fn build(self) -> Result<Catalog, CatalogValidationError> {
        Catalog::new(
            self.metadata,
            self.default_base_id,
            self.regions,
            self.bases,
            self.buildables,
            self.products,
        )
    }
}

fn synthetic_catalog() -> Catalog {
    CatalogInput::valid()
        .build()
        .expect("synthetic catalog should be valid")
}

#[test]
fn valid_catalog_preserves_module_display_order() {
    let catalog = synthetic_catalog();

    assert_eq!(
        catalog
            .regions()
            .iter()
            .map(|definition| definition.id().as_str())
            .collect::<Vec<_>>(),
        ["wuling", "valley"]
    );
    assert_eq!(
        catalog
            .bases()
            .iter()
            .map(|definition| definition.id().as_str())
            .collect::<Vec<_>>(),
        ["main_base", "auxiliary_base"]
    );
    assert_eq!(
        catalog
            .buildables()
            .iter()
            .map(|definition| definition.id().as_str())
            .collect::<Vec<_>>(),
        ["smelter", "battery"]
    );
    assert_eq!(
        catalog
            .products()
            .iter()
            .map(|definition| definition.id().as_str())
            .collect::<Vec<_>>(),
        ["iron", "copper"]
    );
}

#[test]
fn valid_catalog_resolves_its_default_base() {
    let catalog = synthetic_catalog();

    assert_eq!(catalog.default_base().id().as_str(), "main_base");
    assert_eq!(catalog.default_base().display_name(), "Main Base");
    assert_eq!(
        catalog.default_base().bounds(),
        GridSize::new(10, 8).expect("positive bounds")
    );
}

#[test]
fn typed_lookups_return_definitions_from_display_sequences() {
    let catalog = synthetic_catalog();

    assert_eq!(
        catalog.region(&RegionId::new("valley").expect("valid region ID")),
        Some(&catalog.regions()[1])
    );
    assert_eq!(
        catalog.base(&BaseId::new("auxiliary_base").expect("valid base ID")),
        Some(&catalog.bases()[1])
    );
    assert_eq!(
        catalog.buildable(&BuildableId::new("battery").expect("valid buildable ID")),
        Some(&catalog.buildables()[1])
    );
    assert_eq!(
        catalog.product(&ProductId::new("copper").expect("valid product ID")),
        Some(&catalog.products()[1])
    );
}

#[test]
fn catalog_clone_shares_the_immutable_snapshot() {
    let catalog = synthetic_catalog();
    let cloned = catalog.clone();

    assert_eq!(cloned, catalog);
    assert!(std::ptr::eq(cloned.regions(), catalog.regions()));
    assert!(std::ptr::eq(cloned.bases(), catalog.bases()));
    assert!(std::ptr::eq(cloned.buildables(), catalog.buildables()));
    assert!(std::ptr::eq(cloned.products(), catalog.products()));
}

#[test]
fn catalog_equality_compares_independent_snapshot_contents_and_order() {
    let catalog = CatalogInput::valid()
        .build()
        .expect("valid input should build a catalog");
    let independently_built = CatalogInput::valid()
        .build()
        .expect("the same valid input should build another catalog");

    assert!(!std::ptr::eq(
        catalog.regions().as_ptr(),
        independently_built.regions().as_ptr()
    ));
    assert_eq!(catalog, independently_built);

    let mut reordered_input = CatalogInput::valid();
    reordered_input.products.swap(0, 1);
    let reordered = reordered_input
        .build()
        .expect("reordering valid products should remain a valid catalog");

    assert_ne!(catalog, reordered);
}

#[test]
fn duplicate_ids_are_rejected_in_each_namespace() {
    let mut input = CatalogInput::valid();
    let duplicate = input.regions[0].id().clone();
    input
        .regions
        .push(RegionDefinition::new(duplicate.clone(), "Duplicate"));
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::DuplicateRegionId(duplicate))
    );

    let mut input = CatalogInput::valid();
    let duplicate = input.bases[0].id().clone();
    input.bases.push(BaseDefinition::new(
        duplicate.clone(),
        "Duplicate",
        input.regions[0].id().clone(),
        GridSize::new(1, 1).expect("positive bounds"),
    ));
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::DuplicateBaseId(duplicate))
    );

    let mut input = CatalogInput::valid();
    let duplicate = input.buildables[0].id().clone();
    input.buildables.push(BuildableDefinition::new(
        duplicate.clone(),
        "Duplicate",
        CategoryId::new("production").expect("valid category ID"),
        "DP",
        GridSize::new(1, 1).expect("positive footprint"),
        vec![],
    ));
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::DuplicateBuildableId(duplicate))
    );

    let mut input = CatalogInput::valid();
    let duplicate = input.products[0].id().clone();
    input
        .products
        .push(ProductDefinition::new(duplicate.clone(), "Duplicate"));
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::DuplicateProductId(duplicate))
    );
}

#[test]
fn blank_display_names_are_rejected_for_metadata_and_definitions() {
    let mut input = CatalogInput::valid();
    input.metadata = CatalogMetadata::new(
        CatalogId::new("synthetic_catalog").expect("valid catalog ID"),
        Version::parse("1.2.3").expect("valid semantic version"),
        " \t",
    );
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::EmptyCatalogDisplayName)
    );

    let mut input = CatalogInput::valid();
    let id = input.regions[0].id().clone();
    input.regions[0] = RegionDefinition::new(id.clone(), " \n");
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::EmptyRegionDisplayName(id))
    );

    let mut input = CatalogInput::valid();
    let id = input.bases[0].id().clone();
    let region_id = input.bases[0].region_id().clone();
    let bounds = input.bases[0].bounds();
    input.bases[0] = BaseDefinition::new(id.clone(), "", region_id, bounds);
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::EmptyBaseDisplayName(id))
    );

    let mut input = CatalogInput::valid();
    let id = input.buildables[0].id().clone();
    let category_id = input.buildables[0].category_id().clone();
    let symbol = input.buildables[0].symbol().to_owned();
    let footprint = input.buildables[0].footprint();
    let production_targets = input.buildables[0].production_targets().to_vec();
    input.buildables[0] = BuildableDefinition::new(
        id.clone(),
        "  ",
        category_id,
        symbol,
        footprint,
        production_targets,
    );
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::EmptyBuildableDisplayName(id))
    );

    let mut input = CatalogInput::valid();
    let id = input.products[0].id().clone();
    input.products[0] = ProductDefinition::new(id.clone(), "\r\n");
    assert_eq!(
        input.build(),
        Err(CatalogValidationError::EmptyProductDisplayName(id))
    );
}

#[test]
fn buildable_symbol_must_have_one_to_four_trimmed_characters() {
    for invalid_symbol in [" \t", "ABCDE"] {
        let mut input = CatalogInput::valid();
        let original = &input.buildables[0];
        let id = original.id().clone();
        input.buildables[0] = BuildableDefinition::new(
            id.clone(),
            original.display_name(),
            original.category_id().clone(),
            invalid_symbol,
            original.footprint(),
            original.production_targets().to_vec(),
        );

        assert_eq!(
            input.build(),
            Err(CatalogValidationError::InvalidBuildableSymbol(id)),
            "symbol {invalid_symbol:?} must be rejected"
        );
    }
}

#[test]
fn runtime_definition_dimensions_require_validated_grid_sizes() {
    assert!(GridSize::new(0, 1).is_err());
    assert!(GridSize::new(1, 0).is_err());
}

#[test]
fn missing_default_base_is_rejected() {
    let mut input = CatalogInput::valid();
    let missing = BaseId::new("missing_base").expect("valid base ID");
    input.default_base_id = missing.clone();

    assert_eq!(
        input.build(),
        Err(CatalogValidationError::MissingDefaultBase(missing))
    );
}

#[test]
fn base_referencing_a_missing_region_is_rejected() {
    let mut input = CatalogInput::valid();
    let base_id = input.bases[0].id().clone();
    let missing_region_id = RegionId::new("missing_region").expect("valid region ID");
    input.bases[0] = BaseDefinition::new(
        base_id.clone(),
        input.bases[0].display_name(),
        missing_region_id.clone(),
        input.bases[0].bounds(),
    );

    assert_eq!(
        input.build(),
        Err(CatalogValidationError::MissingRegion {
            base_id,
            region_id: missing_region_id,
        })
    );
}

#[test]
fn buildable_referencing_a_missing_product_is_rejected() {
    let mut input = CatalogInput::valid();
    let original = &input.buildables[0];
    let buildable_id = original.id().clone();
    let missing_product_id = ProductId::new("missing_product").expect("valid product ID");
    input.buildables[0] = BuildableDefinition::new(
        buildable_id.clone(),
        original.display_name(),
        original.category_id().clone(),
        original.symbol(),
        original.footprint(),
        vec![missing_product_id.clone()],
    );

    assert_eq!(
        input.build(),
        Err(CatalogValidationError::MissingProductionTarget {
            buildable_id,
            product_id: missing_product_id,
        })
    );
}

#[test]
fn duplicate_production_target_on_one_buildable_is_rejected() {
    let mut input = CatalogInput::valid();
    let original = &input.buildables[0];
    let buildable_id = original.id().clone();
    let product_id = original.production_targets()[0].clone();
    input.buildables[0] = BuildableDefinition::new(
        buildable_id.clone(),
        original.display_name(),
        original.category_id().clone(),
        original.symbol(),
        original.footprint(),
        vec![product_id.clone(), product_id.clone()],
    );

    assert_eq!(
        input.build(),
        Err(CatalogValidationError::DuplicateProductionTarget {
            buildable_id,
            product_id,
        })
    );
}
