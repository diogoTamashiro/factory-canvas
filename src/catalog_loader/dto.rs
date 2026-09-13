use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManifestDto {
    pub(super) schema_version: u64,
    pub(super) catalog_id: String,
    pub(super) data_version: String,
    pub(super) display_name: String,
    pub(super) default_base_id: String,
    pub(super) modules: ModulePathsDto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ModulePathsDto {
    pub(super) regions: String,
    pub(super) bases: String,
    pub(super) buildables: String,
    pub(super) products: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegionsModuleDto {
    pub(super) regions: Vec<RegionDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegionDto {
    pub(super) id: String,
    pub(super) display_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BasesModuleDto {
    pub(super) bases: Vec<BaseDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BaseDto {
    pub(super) id: String,
    pub(super) display_name: String,
    pub(super) region_id: String,
    pub(super) width: u64,
    pub(super) height: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BuildablesModuleDto {
    pub(super) buildables: Vec<BuildableDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BuildableDto {
    pub(super) id: String,
    pub(super) display_name: String,
    pub(super) category: String,
    pub(super) symbol: String,
    pub(super) footprint: DimensionsDto,
    pub(super) production_targets: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProductsModuleDto {
    pub(super) products: Vec<ProductDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProductDto {
    pub(super) id: String,
    pub(super) display_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DimensionsDto {
    pub(super) width: u64,
    pub(super) height: u64,
}
