use super::directory_source::{ensure_module_within_root, CatalogSource, CatalogSourceError};
use super::errors::{CatalogJsonErrorKind, CatalogLoadError, CatalogPathErrorKind};
use super::{load_catalog_from_source, CatalogModule};
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

    let error =
        load_catalog_from_source(&source).expect_err("malformed buildables JSON must be rejected");

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

    for (case, source_file, needle, replacement, expected_module, expected_index, expected_field) in
        cases
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
    source.manifest = source
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
