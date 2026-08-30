use crate::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CatalogValidationError, CategoryId, IdentifierError, ProductDefinition, ProductId,
    RegionDefinition, RegionId,
};
use crate::domain::geometry::{GridSize, GridSizeError};
use semver::Version;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogJsonErrorKind {
    Io,
    Syntax,
    Schema,
    UnexpectedEndOfInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogPathErrorKind {
    Empty,
    Rooted,
    WindowsPrefix,
    CurrentDirectory,
    ParentDirectory,
    EmptyComponent,
    NulByte,
}

impl fmt::Display for CatalogPathErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "is empty",
            Self::Rooted => "is rooted",
            Self::WindowsPrefix => "contains a Windows prefix or alternate stream separator",
            Self::CurrentDirectory => "contains a current-directory component",
            Self::ParentDirectory => "contains a parent-directory component",
            Self::EmptyComponent => "contains an empty component",
            Self::NulByte => "contains a NUL byte",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogLoadError {
    ManifestRead(io::ErrorKind),
    ModuleRead {
        module: CatalogModule,
        kind: io::ErrorKind,
    },
    ModuleOutsideRoot {
        module: CatalogModule,
    },
    InvalidJson {
        module: CatalogModule,
        kind: CatalogJsonErrorKind,
        line: usize,
        column: usize,
    },
    UnsupportedSchemaVersion(u64),
    InvalidDataVersion,
    InvalidIdentifier {
        module: CatalogModule,
        item_index: Option<usize>,
        field: &'static str,
    },
    InvalidDimension {
        module: CatalogModule,
        item_index: usize,
        field: &'static str,
        value: u64,
    },
    InvalidModulePath {
        module: CatalogModule,
        kind: CatalogPathErrorKind,
    },
    DuplicateModulePath {
        first: CatalogModule,
        second: CatalogModule,
    },
    InvalidCatalog(CatalogValidationError),
}

impl CatalogLoadError {
    pub fn is_manifest_not_found(&self) -> bool {
        matches!(self, Self::ManifestRead(io::ErrorKind::NotFound))
    }
}

impl fmt::Display for CatalogLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestRead(kind) => {
                write!(formatter, "The catalog manifest could not be read ({kind}).")
            }
            Self::ModuleRead { module, kind } => {
                write!(
                    formatter,
                    "The {module} catalog module could not be read ({kind:?})."
                )
            }
            Self::ModuleOutsideRoot { module } => write!(
                formatter,
                "The {module} catalog module resolves outside the package root."
            ),
            Self::InvalidJson {
                module,
                kind,
                line,
                column,
            } => {
                let description = match kind {
                    CatalogJsonErrorKind::Io => "could not be decoded",
                    CatalogJsonErrorKind::Syntax => "contains invalid JSON syntax",
                    CatalogJsonErrorKind::Schema => "does not match the expected schema",
                    CatalogJsonErrorKind::UnexpectedEndOfInput => "ends unexpectedly",
                };
                write!(
                    formatter,
                    "The {module} catalog module {description} at line {line}, column {column}."
                )
            }
            Self::UnsupportedSchemaVersion(version) => write!(
                formatter,
                "The catalog schema version {version} is not supported."
            ),
            Self::InvalidDataVersion => {
                formatter.write_str("The catalog data_version is not valid SemVer.")
            }
            Self::InvalidIdentifier {
                module,
                item_index,
                field,
            } => match item_index {
                Some(index) => write!(
                    formatter,
                    "The {field} field in {module} item {} is not a valid ASCII snake_case identifier.",
                    index + 1
                ),
                None => write!(
                    formatter,
                    "The {field} field in the {module} is not a valid ASCII snake_case identifier."
                ),
            },
            Self::InvalidDimension {
                module,
                item_index,
                field,
                value,
            } => write!(
                formatter,
                "The {field} field in {module} item {} has invalid dimension {value}.",
                item_index + 1
            ),
            Self::InvalidModulePath { module, kind } => {
                write!(formatter, "The {module} catalog module path {kind}.")
            }
            Self::DuplicateModulePath { first, second } => write!(
                formatter,
                "The {first} and {second} catalog modules use the same path."
            ),
            Self::InvalidCatalog(error) => {
                write!(formatter, "The catalog failed integrity validation: {error}.")
            }
        }
    }
}

impl std::error::Error for CatalogLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidCatalog(error) => Some(error),
            _ => None,
        }
    }
}

trait CatalogSource {
    fn read_manifest(&self) -> Result<String, io::ErrorKind>;
    fn read_module(&self, path: &str) -> Result<String, CatalogSourceError>;
}

enum CatalogSourceError {
    Io(io::ErrorKind),
    OutsideRoot,
}

struct DirectorySource {
    root: PathBuf,
}

impl DirectorySource {
    fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_owned(),
        }
    }
}

impl CatalogSource for DirectorySource {
    fn read_manifest(&self) -> Result<String, io::ErrorKind> {
        fs::read_to_string(self.root.join("manifest.json")).map_err(|error| error.kind())
    }

    fn read_module(&self, path: &str) -> Result<String, CatalogSourceError> {
        let canonical_root =
            fs::canonicalize(&self.root).map_err(|error| CatalogSourceError::Io(error.kind()))?;
        let canonical_module = fs::canonicalize(self.root.join(path))
            .map_err(|error| CatalogSourceError::Io(error.kind()))?;

        ensure_module_within_root(&canonical_root, &canonical_module)?;

        fs::read_to_string(canonical_module).map_err(|error| CatalogSourceError::Io(error.kind()))
    }
}

fn ensure_module_within_root(
    canonical_root: &Path,
    canonical_module: &Path,
) -> Result<(), CatalogSourceError> {
    if canonical_module.starts_with(canonical_root) {
        Ok(())
    } else {
        Err(CatalogSourceError::OutsideRoot)
    }
}

pub fn load_catalog_from_directory(root: impl AsRef<Path>) -> Result<Catalog, CatalogLoadError> {
    load_catalog_from_source(&DirectorySource::new(root))
}

pub fn load_embedded_public_catalog() -> Result<Catalog, CatalogLoadError> {
    load_catalog_from_source(&EmbeddedPublicSource)
}

struct EmbeddedPublicSource;

impl CatalogSource for EmbeddedPublicSource {
    fn read_manifest(&self) -> Result<String, io::ErrorKind> {
        Ok(include_str!("../catalog/public/manifest.json").to_owned())
    }

    fn read_module(&self, path: &str) -> Result<String, CatalogSourceError> {
        match path {
            "regions.json" => Ok(include_str!("../catalog/public/regions.json").to_owned()),
            "bases.json" => Ok(include_str!("../catalog/public/bases.json").to_owned()),
            "buildables.json" => Ok(include_str!("../catalog/public/buildables.json").to_owned()),
            "products.json" => Ok(include_str!("../catalog/public/products.json").to_owned()),
            _ => Err(CatalogSourceError::Io(io::ErrorKind::NotFound)),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDto {
    schema_version: u64,
    catalog_id: String,
    data_version: String,
    display_name: String,
    default_base_id: String,
    modules: ModulePathsDto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModulePathsDto {
    regions: String,
    bases: String,
    buildables: String,
    products: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionsModuleDto {
    regions: Vec<RegionDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionDto {
    id: String,
    display_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BasesModuleDto {
    bases: Vec<BaseDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BaseDto {
    id: String,
    display_name: String,
    region_id: String,
    width: u64,
    height: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildablesModuleDto {
    buildables: Vec<BuildableDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildableDto {
    id: String,
    display_name: String,
    category: String,
    symbol: String,
    footprint: DimensionsDto,
    production_targets: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductsModuleDto {
    products: Vec<ProductDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductDto {
    id: String,
    display_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DimensionsDto {
    width: u64,
    height: u64,
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

fn read_module(
    source: &impl CatalogSource,
    module: CatalogModule,
    path: &str,
) -> Result<String, CatalogLoadError> {
    source.read_module(path).map_err(|error| match error {
        CatalogSourceError::Io(kind) => CatalogLoadError::ModuleRead { module, kind },
        CatalogSourceError::OutsideRoot => CatalogLoadError::ModuleOutsideRoot { module },
    })
}

fn validate_module_path(raw_path: &str, module: CatalogModule) -> Result<String, CatalogLoadError> {
    let invalid = |kind| CatalogLoadError::InvalidModulePath { module, kind };

    if raw_path.trim().is_empty() {
        return Err(invalid(CatalogPathErrorKind::Empty));
    }
    if raw_path.contains('\0') {
        return Err(invalid(CatalogPathErrorKind::NulByte));
    }

    let normalized = raw_path.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(invalid(CatalogPathErrorKind::Rooted));
    }
    if normalized.contains(':') {
        return Err(invalid(CatalogPathErrorKind::WindowsPrefix));
    }

    for component in normalized.split('/') {
        match component {
            "" => return Err(invalid(CatalogPathErrorKind::EmptyComponent)),
            "." => return Err(invalid(CatalogPathErrorKind::CurrentDirectory)),
            ".." => return Err(invalid(CatalogPathErrorKind::ParentDirectory)),
            _ => {}
        }
    }

    Ok(normalized)
}

fn validate_unique_module_paths(paths: [(CatalogModule, &str); 4]) -> Result<(), CatalogLoadError> {
    for (index, (first_module, first_path)) in paths.iter().enumerate() {
        for (second_module, second_path) in &paths[index + 1..] {
            if first_path == second_path {
                return Err(CatalogLoadError::DuplicateModulePath {
                    first: *first_module,
                    second: *second_module,
                });
            }
        }
    }

    Ok(())
}

fn parse_json<T: DeserializeOwned>(
    text: &str,
    module: CatalogModule,
) -> Result<T, CatalogLoadError> {
    serde_json::from_str(text).map_err(|error| CatalogLoadError::InvalidJson {
        module,
        kind: match error.classify() {
            serde_json::error::Category::Io => CatalogJsonErrorKind::Io,
            serde_json::error::Category::Syntax => CatalogJsonErrorKind::Syntax,
            serde_json::error::Category::Data => CatalogJsonErrorKind::Schema,
            serde_json::error::Category::Eof => CatalogJsonErrorKind::UnexpectedEndOfInput,
        },
        line: error.line(),
        column: error.column(),
    })
}

fn parse_identifier<T>(
    value: String,
    module: CatalogModule,
    item_index: Option<usize>,
    field: &'static str,
    constructor: impl FnOnce(String) -> Result<T, IdentifierError>,
) -> Result<T, CatalogLoadError> {
    constructor(value).map_err(|_| CatalogLoadError::InvalidIdentifier {
        module,
        item_index,
        field,
    })
}

fn parse_dimensions(
    dimensions: DimensionsDto,
    module: CatalogModule,
    item_index: usize,
) -> Result<GridSize, CatalogLoadError> {
    let width =
        u16::try_from(dimensions.width).map_err(|_| CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field: "width",
            value: dimensions.width,
        })?;
    let height =
        u16::try_from(dimensions.height).map_err(|_| CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field: "height",
            value: dimensions.height,
        })?;

    GridSize::new(width, height).map_err(|error| {
        let (field, value) = match error {
            GridSizeError::ZeroWidth => ("width", dimensions.width),
            GridSizeError::ZeroHeight => ("height", dimensions.height),
        };
        CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field,
            value,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_module_within_root, load_catalog_from_source, CatalogJsonErrorKind,
        CatalogLoadError, CatalogModule, CatalogPathErrorKind, CatalogSource, CatalogSourceError,
    };
    use crate::domain::catalog::{
        BaseId, BuildableId, CatalogValidationError, IdentifierError, ProductId, RegionId,
    };
    use std::collections::BTreeMap;
    use std::io;
    use std::path::Path;

    struct MemorySource {
        manifest: String,
        modules: BTreeMap<String, String>,
    }

    impl CatalogSource for MemorySource {
        fn read_manifest(&self) -> Result<String, io::ErrorKind> {
            Ok(self.manifest.clone())
        }

        fn read_module(&self, path: &str) -> Result<String, CatalogSourceError> {
            self.modules
                .get(path)
                .cloned()
                .ok_or(CatalogSourceError::Io(io::ErrorKind::NotFound))
        }
    }

    fn valid_memory_source() -> MemorySource {
        let manifest = r#"{
            "schema_version": 1,
            "catalog_id": "test_catalog",
            "data_version": "2.3.4",
            "display_name": "Test Catalog",
            "default_base_id": "test_base",
            "modules": {
                "regions": "regions.json",
                "bases": "bases.json",
                "buildables": "buildables.json",
                "products": "products.json"
            }
        }"#;
        let regions = r#"{
            "regions": [
                { "id": "test_region", "display_name": "Test Region" }
            ]
        }"#;
        let bases = r#"{
            "bases": [
                {
                    "id": "test_base",
                    "display_name": "Test Base",
                    "region_id": "test_region",
                    "width": 7,
                    "height": 5
                }
            ]
        }"#;
        let buildables = r#"{
            "buildables": [
                {
                    "id": "test_machine",
                    "display_name": "Test Machine",
                    "category": "production",
                    "symbol": "TM",
                    "footprint": { "width": 2, "height": 3 },
                    "production_targets": ["test_product"]
                }
            ]
        }"#;
        let products = r#"{
            "products": [
                { "id": "test_product", "display_name": "Test Product" }
            ]
        }"#;

        MemorySource {
            manifest: manifest.to_owned(),
            modules: [
                ("regions.json".to_owned(), regions.to_owned()),
                ("bases.json".to_owned(), bases.to_owned()),
                ("buildables.json".to_owned(), buildables.to_owned()),
                ("products.json".to_owned(), products.to_owned()),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn valid_memory_package_loads_a_complete_catalog() {
        let catalog = load_catalog_from_source(&valid_memory_source())
            .expect("the complete synthetic catalog should load");

        assert_eq!(catalog.metadata().catalog_id().as_str(), "test_catalog");
        assert_eq!(catalog.metadata().data_version().to_string(), "2.3.4");
        assert_eq!(catalog.default_base().id().as_str(), "test_base");
        assert_eq!(catalog.default_base().bounds().width(), 7);
        assert_eq!(catalog.default_base().bounds().height(), 5);
        assert_eq!(catalog.regions()[0].id().as_str(), "test_region");
        assert_eq!(catalog.buildables()[0].id().as_str(), "test_machine");
        assert_eq!(catalog.buildables()[0].footprint().width(), 2);
        assert_eq!(catalog.buildables()[0].footprint().height(), 3);
        assert_eq!(
            catalog.buildables()[0].production_targets()[0].as_str(),
            "test_product"
        );
        assert_eq!(catalog.products()[0].id().as_str(), "test_product");
    }

    #[test]
    fn malformed_buildables_json_returns_a_positioned_error() {
        let mut source = valid_memory_source();
        source.modules.insert(
            "buildables.json".to_owned(),
            "{ \"buildables\": [ {".to_owned(),
        );

        let error = load_catalog_from_source(&source)
            .expect_err("malformed buildables JSON must be rejected");

        let CatalogLoadError::InvalidJson {
            module,
            kind,
            line,
            column,
        } = &error
        else {
            panic!("expected an InvalidJson error, got {error:?}");
        };
        assert_eq!(*module, CatalogModule::Buildables);
        assert_eq!(*kind, CatalogJsonErrorKind::UnexpectedEndOfInput);
        assert!(*line > 0);
        assert!(*column > 0);

        let message = error.to_string();
        assert!(message.contains("buildables"));
        assert!(message.contains(&format!("line {line}")));
        assert!(message.contains(&format!("column {column}")));
        assert!(!message.contains("\"buildables\""));
    }

    #[test]
    fn malformed_buildable_objects_return_schema_errors_without_echoing_values() {
        let cases = [
            (
                "missing footprint",
                r#"{
                    "buildables": [{
                        "id": "test_machine",
                        "display_name": "Test Machine",
                        "category": "production",
                        "symbol": "TM",
                        "production_targets": ["test_product"]
                    }]
                }"#,
            ),
            (
                "unknown field",
                r#"{
                    "buildables": [{
                        "id": "test_machine",
                        "display_name": "Test Machine",
                        "category": "production",
                        "symbol": "TM",
                        "footprint": { "width": 2, "height": 3 },
                        "production_targets": ["test_product"],
                        "private_sentinel": "do-not-echo-this-value"
                    }]
                }"#,
            ),
            (
                "wrong field type",
                r#"{
                    "buildables": [{
                        "id": "test_machine",
                        "display_name": "Test Machine",
                        "category": "production",
                        "symbol": "TM",
                        "footprint": {
                            "width": "do-not-echo-this-value",
                            "height": 3
                        },
                        "production_targets": ["test_product"]
                    }]
                }"#,
            ),
        ];

        for (case, buildables) in cases {
            let mut source = valid_memory_source();
            source
                .modules
                .insert("buildables.json".to_owned(), buildables.to_owned());

            let error = match load_catalog_from_source(&source) {
                Ok(_) => panic!("{case} must be rejected"),
                Err(error) => error,
            };
            let CatalogLoadError::InvalidJson {
                module,
                kind,
                line,
                column,
            } = &error
            else {
                panic!("{case}: expected InvalidJson, got {error:?}");
            };
            assert_eq!(*module, CatalogModule::Buildables, "{case}");
            assert_eq!(*kind, CatalogJsonErrorKind::Schema, "{case}");
            assert!(*line > 0, "{case}");
            assert!(*column > 0, "{case}");
            assert!(!error.to_string().contains("do-not-echo-this-value"));
        }
    }

    #[test]
    fn unknown_fields_are_rejected_at_every_catalog_object_boundary() {
        #[derive(Clone, Copy)]
        enum SourceFile {
            Manifest,
            Module(&'static str),
        }

        let cases = [
            (
                "manifest",
                SourceFile::Manifest,
                CatalogModule::Manifest,
                r#""schema_version": 1,"#,
                r#""schema_version": 1, "unexpected": true,"#,
            ),
            (
                "module paths",
                SourceFile::Manifest,
                CatalogModule::Manifest,
                r#""regions": "regions.json","#,
                r#""regions": "regions.json", "unexpected": "ignored.json","#,
            ),
            (
                "regions wrapper",
                SourceFile::Module("regions.json"),
                CatalogModule::Regions,
                r#""regions": ["#,
                r#""unexpected": true, "regions": ["#,
            ),
            (
                "region item",
                SourceFile::Module("regions.json"),
                CatalogModule::Regions,
                r#""id": "test_region","#,
                r#""id": "test_region", "unexpected": true,"#,
            ),
            (
                "bases wrapper",
                SourceFile::Module("bases.json"),
                CatalogModule::Bases,
                r#""bases": ["#,
                r#""unexpected": true, "bases": ["#,
            ),
            (
                "base item",
                SourceFile::Module("bases.json"),
                CatalogModule::Bases,
                r#""id": "test_base","#,
                r#""id": "test_base", "unexpected": true,"#,
            ),
            (
                "buildables wrapper",
                SourceFile::Module("buildables.json"),
                CatalogModule::Buildables,
                r#""buildables": ["#,
                r#""unexpected": true, "buildables": ["#,
            ),
            (
                "buildable footprint",
                SourceFile::Module("buildables.json"),
                CatalogModule::Buildables,
                r#""width": 2, "height": 3"#,
                r#""width": 2, "height": 3, "unexpected": 1"#,
            ),
            (
                "products wrapper",
                SourceFile::Module("products.json"),
                CatalogModule::Products,
                r#""products": ["#,
                r#""unexpected": true, "products": ["#,
            ),
            (
                "product item",
                SourceFile::Module("products.json"),
                CatalogModule::Products,
                r#""id": "test_product","#,
                r#""id": "test_product", "unexpected": true,"#,
            ),
        ];

        for (case, source_file, expected_module, needle, replacement) in cases {
            let mut source = valid_memory_source();
            match source_file {
                SourceFile::Manifest => {
                    let changed = source.manifest.replacen(needle, replacement, 1);
                    assert_ne!(changed, source.manifest, "invalid test fixture: {case}");
                    source.manifest = changed;
                }
                SourceFile::Module(path) => {
                    let original = source.modules.get(path).unwrap().clone();
                    let changed = original.replacen(needle, replacement, 1);
                    assert_ne!(changed, original, "invalid test fixture: {case}");
                    source.modules.insert(path.to_owned(), changed);
                }
            }

            let error = match load_catalog_from_source(&source) {
                Ok(_) => panic!("{case} must reject unknown fields"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error,
                    CatalogLoadError::InvalidJson {
                        module,
                        kind: CatalogJsonErrorKind::Schema,
                        ..
                    } if module == expected_module
                ),
                "{case}: got {error:?}"
            );
        }
    }

    #[test]
    fn manifest_and_all_module_paths_are_required() {
        enum Location {
            Manifest,
            Modules,
        }

        let cases = [
            (Location::Manifest, "schema_version"),
            (Location::Manifest, "catalog_id"),
            (Location::Manifest, "data_version"),
            (Location::Manifest, "display_name"),
            (Location::Manifest, "default_base_id"),
            (Location::Manifest, "modules"),
            (Location::Modules, "regions"),
            (Location::Modules, "bases"),
            (Location::Modules, "buildables"),
            (Location::Modules, "products"),
        ];

        for (location, field) in cases {
            let mut source = valid_memory_source();
            let mut manifest: serde_json::Value = serde_json::from_str(&source.manifest).unwrap();
            match location {
                Location::Manifest => {
                    manifest.as_object_mut().unwrap().remove(field);
                }
                Location::Modules => {
                    manifest["modules"].as_object_mut().unwrap().remove(field);
                }
            }
            source.manifest = serde_json::to_string(&manifest).unwrap();

            let error = match load_catalog_from_source(&source) {
                Ok(_) => panic!("missing manifest field {field} was accepted"),
                Err(error) => error,
            };

            assert!(matches!(
                error,
                CatalogLoadError::InvalidJson {
                    module: CatalogModule::Manifest,
                    kind: CatalogJsonErrorKind::Schema,
                    ..
                }
            ));
        }
    }

    #[test]
    fn unsupported_manifest_schema_version_is_rejected() {
        let mut source = valid_memory_source();
        source.manifest =
            source
                .manifest
                .replacen(r#""schema_version": 1"#, r#""schema_version": 2"#, 1);

        let error = load_catalog_from_source(&source).unwrap_err();

        assert_eq!(error, CatalogLoadError::UnsupportedSchemaVersion(2));
        assert!(error.to_string().contains("schema version 2"));
    }

    #[test]
    fn invalid_data_version_is_rejected_without_echoing_its_value() {
        let mut source = valid_memory_source();
        source.manifest = source.manifest.replacen(
            r#""data_version": "2.3.4""#,
            r#""data_version": "do-not-echo-this-version""#,
            1,
        );

        let error = load_catalog_from_source(&source).unwrap_err();

        assert_eq!(error, CatalogLoadError::InvalidDataVersion);
        assert!(!error.to_string().contains("do-not-echo-this-version"));
    }

    #[test]
    fn invalid_identifiers_report_their_catalog_location_without_echoing_values() {
        #[derive(Clone, Copy)]
        enum SourceFile {
            Manifest,
            Module(&'static str),
        }

        let cases = [
            (
                "catalog id",
                SourceFile::Manifest,
                r#""catalog_id": "test_catalog""#,
                r#""catalog_id": "Do-Not-Echo""#,
                CatalogModule::Manifest,
                None,
                "catalog_id",
            ),
            (
                "default base",
                SourceFile::Manifest,
                r#""default_base_id": "test_base""#,
                r#""default_base_id": "Do-Not-Echo""#,
                CatalogModule::Manifest,
                None,
                "default_base_id",
            ),
            (
                "region id",
                SourceFile::Module("regions.json"),
                r#""id": "test_region""#,
                r#""id": "Do-Not-Echo""#,
                CatalogModule::Regions,
                Some(0),
                "id",
            ),
            (
                "base id",
                SourceFile::Module("bases.json"),
                r#""id": "test_base""#,
                r#""id": "Do-Not-Echo""#,
                CatalogModule::Bases,
                Some(0),
                "id",
            ),
            (
                "base region",
                SourceFile::Module("bases.json"),
                r#""region_id": "test_region""#,
                r#""region_id": "Do-Not-Echo""#,
                CatalogModule::Bases,
                Some(0),
                "region_id",
            ),
            (
                "buildable id",
                SourceFile::Module("buildables.json"),
                r#""id": "test_machine""#,
                r#""id": "Do-Not-Echo""#,
                CatalogModule::Buildables,
                Some(0),
                "id",
            ),
            (
                "buildable category",
                SourceFile::Module("buildables.json"),
                r#""category": "production""#,
                r#""category": "Do-Not-Echo""#,
                CatalogModule::Buildables,
                Some(0),
                "category",
            ),
            (
                "buildable production target",
                SourceFile::Module("buildables.json"),
                r#""production_targets": ["test_product"]"#,
                r#""production_targets": ["Do-Not-Echo"]"#,
                CatalogModule::Buildables,
                Some(0),
                "production_targets",
            ),
            (
                "product id",
                SourceFile::Module("products.json"),
                r#""id": "test_product""#,
                r#""id": "Do-Not-Echo""#,
                CatalogModule::Products,
                Some(0),
                "id",
            ),
        ];

        for (
            case,
            source_file,
            needle,
            replacement,
            expected_module,
            expected_index,
            expected_field,
        ) in cases
        {
            let mut source = valid_memory_source();
            match source_file {
                SourceFile::Manifest => {
                    source.manifest = source.manifest.replacen(needle, replacement, 1);
                }
                SourceFile::Module(path) => {
                    let text = source.modules.get(path).unwrap();
                    source
                        .modules
                        .insert(path.to_owned(), text.replacen(needle, replacement, 1));
                }
            }

            let error = load_catalog_from_source(&source).unwrap_err();

            assert_eq!(
                error,
                CatalogLoadError::InvalidIdentifier {
                    module: expected_module,
                    item_index: expected_index,
                    field: expected_field,
                },
                "{case}"
            );
            assert!(!error.to_string().contains("Do-Not-Echo"), "{case}");
        }
    }

    #[test]
    fn zero_and_overflow_dimensions_report_their_catalog_location() {
        let cases = [
            (
                "base width zero",
                "bases.json",
                r#""width": 7"#,
                r#""width": 0"#,
                CatalogModule::Bases,
                "width",
                0,
            ),
            (
                "base height zero",
                "bases.json",
                r#""height": 5"#,
                r#""height": 0"#,
                CatalogModule::Bases,
                "height",
                0,
            ),
            (
                "base width overflow",
                "bases.json",
                r#""width": 7"#,
                r#""width": 65536"#,
                CatalogModule::Bases,
                "width",
                65_536,
            ),
            (
                "base height overflow",
                "bases.json",
                r#""height": 5"#,
                r#""height": 65536"#,
                CatalogModule::Bases,
                "height",
                65_536,
            ),
            (
                "buildable width zero",
                "buildables.json",
                r#""width": 2"#,
                r#""width": 0"#,
                CatalogModule::Buildables,
                "width",
                0,
            ),
            (
                "buildable height zero",
                "buildables.json",
                r#""height": 3"#,
                r#""height": 0"#,
                CatalogModule::Buildables,
                "height",
                0,
            ),
            (
                "buildable width overflow",
                "buildables.json",
                r#""width": 2"#,
                r#""width": 65536"#,
                CatalogModule::Buildables,
                "width",
                65_536,
            ),
            (
                "buildable height overflow",
                "buildables.json",
                r#""height": 3"#,
                r#""height": 65536"#,
                CatalogModule::Buildables,
                "height",
                65_536,
            ),
        ];

        for (case, path, needle, replacement, module, field, value) in cases {
            let mut source = valid_memory_source();
            let text = source.modules.get(path).unwrap();
            source
                .modules
                .insert(path.to_owned(), text.replacen(needle, replacement, 1));

            let error = load_catalog_from_source(&source).unwrap_err();

            assert_eq!(
                error,
                CatalogLoadError::InvalidDimension {
                    module,
                    item_index: 0,
                    field,
                    value,
                },
                "{case}"
            );
        }
    }

    #[test]
    fn catalog_integrity_failures_are_returned_without_partial_catalogs() {
        #[derive(Clone, Copy)]
        enum SourceFile {
            Manifest,
            Module(&'static str),
        }

        let cases = vec![
            (
                "duplicate region",
                SourceFile::Module("regions.json"),
                r#"{"regions":[{"id":"test_region","display_name":"One"},{"id":"test_region","display_name":"Two"}]}"#,
                CatalogValidationError::DuplicateRegionId(RegionId::new("test_region").unwrap()),
            ),
            (
                "blank buildable name",
                SourceFile::Module("buildables.json"),
                r#"{"buildables":[{"id":"test_machine","display_name":"   ","category":"production","symbol":"TM","footprint":{"width":2,"height":3},"production_targets":["test_product"]}]}"#,
                CatalogValidationError::EmptyBuildableDisplayName(
                    BuildableId::new("test_machine").unwrap(),
                ),
            ),
            (
                "invalid buildable symbol",
                SourceFile::Module("buildables.json"),
                r#"{"buildables":[{"id":"test_machine","display_name":"Test Machine","category":"production","symbol":"TOO-LONG","footprint":{"width":2,"height":3},"production_targets":["test_product"]}]}"#,
                CatalogValidationError::InvalidBuildableSymbol(
                    BuildableId::new("test_machine").unwrap(),
                ),
            ),
            (
                "missing default base",
                SourceFile::Manifest,
                "missing_base",
                CatalogValidationError::MissingDefaultBase(BaseId::new("missing_base").unwrap()),
            ),
            (
                "missing region",
                SourceFile::Module("bases.json"),
                r#"{"bases":[{"id":"test_base","display_name":"Test Base","region_id":"missing_region","width":7,"height":5}]}"#,
                CatalogValidationError::MissingRegion {
                    base_id: BaseId::new("test_base").unwrap(),
                    region_id: RegionId::new("missing_region").unwrap(),
                },
            ),
            (
                "missing product",
                SourceFile::Module("buildables.json"),
                r#"{"buildables":[{"id":"test_machine","display_name":"Test Machine","category":"production","symbol":"TM","footprint":{"width":2,"height":3},"production_targets":["missing_product"]}]}"#,
                CatalogValidationError::MissingProductionTarget {
                    buildable_id: BuildableId::new("test_machine").unwrap(),
                    product_id: ProductId::new("missing_product").unwrap(),
                },
            ),
            (
                "duplicate production target",
                SourceFile::Module("buildables.json"),
                r#"{"buildables":[{"id":"test_machine","display_name":"Test Machine","category":"production","symbol":"TM","footprint":{"width":2,"height":3},"production_targets":["test_product","test_product"]}]}"#,
                CatalogValidationError::DuplicateProductionTarget {
                    buildable_id: BuildableId::new("test_machine").unwrap(),
                    product_id: ProductId::new("test_product").unwrap(),
                },
            ),
        ];

        for (case, source_file, replacement, expected) in cases {
            let mut source = valid_memory_source();
            match source_file {
                SourceFile::Manifest => {
                    source.manifest = source.manifest.replacen("test_base", replacement, 1);
                }
                SourceFile::Module(path) => {
                    source
                        .modules
                        .insert(path.to_owned(), replacement.to_owned());
                }
            }

            let error = load_catalog_from_source(&source).unwrap_err();

            assert_eq!(error, CatalogLoadError::InvalidCatalog(expected), "{case}");
        }
    }

    #[test]
    fn standard_errors_chain_catalog_validation_context() {
        fn assert_standard_error<T: std::error::Error>() {}

        assert_standard_error::<IdentifierError>();
        assert_standard_error::<CatalogValidationError>();
        assert_standard_error::<CatalogLoadError>();

        let error = CatalogLoadError::InvalidCatalog(CatalogValidationError::MissingDefaultBase(
            BaseId::new("missing_base").unwrap(),
        ));
        let source = std::error::Error::source(&error).expect("validation source must be retained");

        assert!(source.to_string().contains("missing_base"));
        assert!(source.to_string().contains("default base"));
        assert!(error.to_string().contains(&source.to_string()));
    }

    #[test]
    fn unsafe_module_paths_are_rejected_without_echoing_them() {
        let cases = [
            ("\"\"", CatalogPathErrorKind::Empty),
            (r#""   ""#, CatalogPathErrorKind::Empty),
            (r#""/absolute/private.json""#, CatalogPathErrorKind::Rooted),
            (r#""\\rooted.json""#, CatalogPathErrorKind::Rooted),
            (r#""C:/private.json""#, CatalogPathErrorKind::WindowsPrefix),
            (r#""C:\\private.json""#, CatalogPathErrorKind::WindowsPrefix),
            (
                r#""\\\\private-server\\share.json""#,
                CatalogPathErrorKind::Rooted,
            ),
            (
                r#""../regions.json""#,
                CatalogPathErrorKind::ParentDirectory,
            ),
            (
                r#""nested/../regions.json""#,
                CatalogPathErrorKind::ParentDirectory,
            ),
            (
                r#""./regions.json""#,
                CatalogPathErrorKind::CurrentDirectory,
            ),
            (
                r#""nested/./regions.json""#,
                CatalogPathErrorKind::CurrentDirectory,
            ),
            (
                r#""nested//regions.json""#,
                CatalogPathErrorKind::EmptyComponent,
            ),
            (r#""bad\u0000name.json""#, CatalogPathErrorKind::NulByte),
        ];

        for (replacement, expected_kind) in cases {
            let mut source = valid_memory_source();
            source.manifest = source
                .manifest
                .replacen(r#""regions.json""#, replacement, 1);

            let error = load_catalog_from_source(&source).unwrap_err();

            assert_eq!(
                error,
                CatalogLoadError::InvalidModulePath {
                    module: CatalogModule::Regions,
                    kind: expected_kind,
                },
                "{replacement}"
            );
            assert!(!error.to_string().contains("private"), "{replacement}");
        }
    }

    #[test]
    fn safe_relative_backslashes_are_normalized_before_reading() {
        let mut source = valid_memory_source();
        source.manifest =
            source
                .manifest
                .replacen(r#""regions.json""#, r#""nested\\regions.json""#, 1);
        let regions = source.modules.remove("regions.json").unwrap();
        source
            .modules
            .insert("nested/regions.json".to_owned(), regions);

        let catalog = load_catalog_from_source(&source).unwrap();

        assert_eq!(catalog.regions()[0].id().as_str(), "test_region");
    }

    #[test]
    fn repeated_normalized_module_paths_are_rejected_before_reading() {
        let mut source = valid_memory_source();
        source.manifest = source
            .manifest
            .replacen(r#""bases.json""#, r#""regions.json""#, 1);

        let error = load_catalog_from_source(&source).unwrap_err();

        assert_eq!(
            error,
            CatalogLoadError::DuplicateModulePath {
                first: CatalogModule::Regions,
                second: CatalogModule::Bases,
            }
        );
        assert!(!error.to_string().contains("nested/regions.json"));
    }

    #[test]
    fn resolved_module_boundary_is_component_aware() {
        let root = Path::new("package-root");

        assert!(ensure_module_within_root(root, Path::new("package-root/regions.json")).is_ok());
        assert!(matches!(
            ensure_module_within_root(root, Path::new("package-root-sibling/regions.json")),
            Err(CatalogSourceError::OutsideRoot)
        ));
        assert!(matches!(
            ensure_module_within_root(root, Path::new("outside/regions.json")),
            Err(CatalogSourceError::OutsideRoot)
        ));
    }
}
