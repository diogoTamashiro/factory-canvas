mod directory_source;
mod dto;
mod errors;
mod parsing;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;

pub use errors::{CatalogJsonErrorKind, CatalogLoadError, CatalogPathErrorKind};

use directory_source::{CatalogSource, DirectorySource, EmbeddedPublicSource};
use dto::{
    BasesModuleDto, BuildablesModuleDto, DimensionsDto, ManifestDto, ProductsModuleDto,
    RegionsModuleDto,
};
use parsing::{
    parse_dimensions, parse_identifier, parse_json, read_module, validate_module_path,
    validate_unique_module_paths,
};

use crate::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CategoryId, ProductDefinition, ProductId, RegionDefinition, RegionId,
};
use semver::Version;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogModule {
    Manifest,
    Regions,
    Bases,
    Buildables,
    Products,
}

impl fmt::Display for CatalogModule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Manifest => "manifest",
            Self::Regions => "regions",
            Self::Bases => "bases",
            Self::Buildables => "buildables",
            Self::Products => "products",
        })
    }
}

pub fn load_catalog_from_directory(root: impl AsRef<Path>) -> Result<Catalog, CatalogLoadError> {
    load_catalog_from_source(&DirectorySource::new(root))
}

pub fn load_embedded_public_catalog() -> Result<Catalog, CatalogLoadError> {
    load_catalog_from_source(&EmbeddedPublicSource)
}

fn load_catalog_from_source(source: &impl CatalogSource) -> Result<Catalog, CatalogLoadError> {
    let manifest_text = source
        .read_manifest()
        .map_err(CatalogLoadError::ManifestRead)?;
    let manifest: ManifestDto = parse_json(&manifest_text, CatalogModule::Manifest)?;
    if manifest.schema_version != 1 {
        return Err(CatalogLoadError::UnsupportedSchemaVersion(
            manifest.schema_version,
        ));
    }

    let regions_path = validate_module_path(&manifest.modules.regions, CatalogModule::Regions)?;
    let bases_path = validate_module_path(&manifest.modules.bases, CatalogModule::Bases)?;
    let buildables_path =
        validate_module_path(&manifest.modules.buildables, CatalogModule::Buildables)?;
    let products_path = validate_module_path(&manifest.modules.products, CatalogModule::Products)?;
    validate_unique_module_paths([
        (CatalogModule::Regions, regions_path.as_str()),
        (CatalogModule::Bases, bases_path.as_str()),
        (CatalogModule::Buildables, buildables_path.as_str()),
        (CatalogModule::Products, products_path.as_str()),
    ])?;

    let regions_text = read_module(source, CatalogModule::Regions, &regions_path)?;
    let bases_text = read_module(source, CatalogModule::Bases, &bases_path)?;
    let buildables_text = read_module(source, CatalogModule::Buildables, &buildables_path)?;
    let products_text = read_module(source, CatalogModule::Products, &products_path)?;

    let regions: RegionsModuleDto = parse_json(&regions_text, CatalogModule::Regions)?;
    let bases: BasesModuleDto = parse_json(&bases_text, CatalogModule::Bases)?;
    let buildables: BuildablesModuleDto = parse_json(&buildables_text, CatalogModule::Buildables)?;
    let products: ProductsModuleDto = parse_json(&products_text, CatalogModule::Products)?;

    let metadata = CatalogMetadata::new(
        parse_identifier(
            manifest.catalog_id,
            CatalogModule::Manifest,
            None,
            "catalog_id",
            CatalogId::new,
        )?,
        Version::parse(&manifest.data_version).map_err(|_| CatalogLoadError::InvalidDataVersion)?,
        manifest.display_name,
    );
    let default_base_id = parse_identifier(
        manifest.default_base_id,
        CatalogModule::Manifest,
        None,
        "default_base_id",
        BaseId::new,
    )?;

    let regions = regions
        .regions
        .into_iter()
        .enumerate()
        .map(|(index, region)| {
            Ok(RegionDefinition::new(
                parse_identifier(
                    region.id,
                    CatalogModule::Regions,
                    Some(index),
                    "id",
                    RegionId::new,
                )?,
                region.display_name,
            ))
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;

    let bases = bases
        .bases
        .into_iter()
        .enumerate()
        .map(|(index, base)| {
            Ok(BaseDefinition::new(
                parse_identifier(
                    base.id,
                    CatalogModule::Bases,
                    Some(index),
                    "id",
                    BaseId::new,
                )?,
                base.display_name,
                parse_identifier(
                    base.region_id,
                    CatalogModule::Bases,
                    Some(index),
                    "region_id",
                    RegionId::new,
                )?,
                parse_dimensions(
                    DimensionsDto {
                        width: base.width,
                        height: base.height,
                    },
                    CatalogModule::Bases,
                    index,
                )?,
            ))
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;

    let buildables = buildables
        .buildables
        .into_iter()
        .enumerate()
        .map(|(index, buildable)| {
            let production_targets = buildable
                .production_targets
                .into_iter()
                .map(|target| {
                    parse_identifier(
                        target,
                        CatalogModule::Buildables,
                        Some(index),
                        "production_targets",
                        ProductId::new,
                    )
                })
                .collect::<Result<Vec<_>, CatalogLoadError>>()?;

            Ok(BuildableDefinition::new(
                parse_identifier(
                    buildable.id,
                    CatalogModule::Buildables,
                    Some(index),
                    "id",
                    BuildableId::new,
                )?,
                buildable.display_name,
                parse_identifier(
                    buildable.category,
                    CatalogModule::Buildables,
                    Some(index),
                    "category",
                    CategoryId::new,
                )?,
                buildable.symbol,
                parse_dimensions(buildable.footprint, CatalogModule::Buildables, index)?,
                production_targets,
                buildable.icon.as_deref(),
            ))
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;

    let products = products
        .products
        .into_iter()
        .enumerate()
        .map(|(index, product)| {
            Ok(ProductDefinition::new(
                parse_identifier(
                    product.id,
                    CatalogModule::Products,
                    Some(index),
                    "id",
                    ProductId::new,
                )?,
                product.display_name,
            ))
        })
        .collect::<Result<Vec<_>, CatalogLoadError>>()?;

    Catalog::new(
        metadata,
        default_base_id,
        regions,
        bases,
        buildables,
        products,
    )
    .map_err(CatalogLoadError::InvalidCatalog)
}
