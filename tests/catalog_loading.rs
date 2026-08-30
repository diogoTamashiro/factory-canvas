use factory_canvas::catalog_loader::{
    load_catalog_from_directory, load_embedded_public_catalog, CatalogLoadError,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let unique = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "factory-canvas-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("test directory must be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn with_public_package(label: &str) -> Self {
        let directory = Self::new(label);
        let public_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog/public");
        for file_name in [
            "manifest.json",
            "regions.json",
            "bases.json",
            "buildables.json",
            "products.json",
        ] {
            fs::copy(
                public_root.join(file_name),
                directory.path().join(file_name),
            )
            .expect("public test module must be copied");
        }
        directory
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[test]
fn missing_directory_manifest_returns_a_safe_not_found_error() {
    let directory = TestDirectory::new("missing-manifest");

    let error = load_catalog_from_directory(directory.path()).unwrap_err();

    assert_eq!(
        error,
        CatalogLoadError::ManifestRead(std::io::ErrorKind::NotFound)
    );
    assert!(error.is_manifest_not_found());
    assert!(!error
        .to_string()
        .contains(&directory.path().display().to_string()));
}

#[test]
fn versioned_public_directory_matches_the_approved_compatibility_catalog() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog/public");
    let catalog = load_catalog_from_directory(root).expect("the public catalog must be valid");

    assert_eq!(
        catalog.metadata().catalog_id().as_str(),
        "factory_canvas_public"
    );
    assert_eq!(catalog.metadata().data_version().to_string(), "0.1.0");
    assert_eq!(
        catalog.metadata().display_name(),
        "Factory Canvas — Public Catalog"
    );
    assert_eq!(catalog.default_base().id().as_str(), "wuling_main");

    assert_eq!(
        catalog
            .regions()
            .iter()
            .map(|region| (region.id().as_str(), region.display_name()))
            .collect::<Vec<_>>(),
        [("wuling", "Wuling")]
    );
    assert_eq!(
        catalog
            .bases()
            .iter()
            .map(|base| (
                base.id().as_str(),
                base.display_name(),
                base.region_id().as_str(),
                base.bounds().width(),
                base.bounds().height(),
            ))
            .collect::<Vec<_>>(),
        [
            ("wuling_main", "Main PAC", "wuling", 80, 80),
            ("wuling_sub_standard", "Standard Sub-PAC", "wuling", 30, 30,),
            (
                "wuling_sub_area_expansion_i",
                "Sub-PAC Expansion I",
                "wuling",
                40,
                40,
            ),
            (
                "wuling_sub_area_expansion_ii",
                "Sub-PAC Expansion II",
                "wuling",
                50,
                50,
            ),
        ]
    );
    assert_eq!(
        catalog
            .buildables()
            .iter()
            .map(|buildable| (
                buildable.id().as_str(),
                buildable.display_name(),
                buildable.category_id().as_str(),
                buildable.symbol(),
                buildable.footprint().width(),
                buildable.footprint().height(),
                buildable.production_targets().len(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                "xiranite_power_pole",
                "Xiranite Power Pole",
                "energy",
                "XPP",
                2,
                2,
                0,
            ),
            (
                "refinery_unit",
                "Refinery Unit",
                "production_i",
                "RU",
                3,
                3,
                0,
            ),
            (
                "crushing_unit",
                "Crushing Unit",
                "production_i",
                "CU",
                3,
                3,
                0,
            ),
        ]
    );
    assert!(catalog.products().is_empty());
}

#[test]
fn embedded_public_catalog_matches_the_versioned_directory_snapshot() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("catalog/public");
    let directory = load_catalog_from_directory(root).expect("directory catalog must load");
    let embedded = load_embedded_public_catalog().expect("embedded catalog must load");

    assert_eq!(embedded, directory);
}

#[test]
fn missing_directory_module_is_an_error_without_embedded_mixing() {
    let directory = TestDirectory::with_public_package("missing-module");
    fs::remove_file(directory.path().join("buildables.json"))
        .expect("buildables module must be removed");

    let error = load_catalog_from_directory(directory.path()).unwrap_err();

    assert_eq!(
        error,
        CatalogLoadError::ModuleRead {
            module: factory_canvas::catalog_loader::CatalogModule::Buildables,
            kind: std::io::ErrorKind::NotFound,
        }
    );
}

#[test]
fn unreadable_directory_module_is_returned_as_a_module_error() {
    let directory = TestDirectory::with_public_package("unreadable-module");
    let buildables_path = directory.path().join("buildables.json");
    fs::remove_file(&buildables_path).expect("buildables file must be removed");
    fs::create_dir(&buildables_path).expect("directory must replace buildables file");

    let error = load_catalog_from_directory(directory.path()).unwrap_err();
    let CatalogLoadError::ModuleRead { module, kind } = error else {
        panic!("inaccessible module must return ModuleRead");
    };

    assert_eq!(
        module,
        factory_canvas::catalog_loader::CatalogModule::Buildables
    );
    assert_ne!(kind, std::io::ErrorKind::NotFound);
}

#[test]
fn malformed_directory_buildable_returns_a_safe_error_instead_of_panicking() {
    let directory = TestDirectory::with_public_package("malformed-buildable");
    fs::write(
        directory.path().join("buildables.json"),
        r#"{
  "buildables": [
    {
      "id": "test_machine",
      "display_name": "Test Machine",
      "category": "production",
      "symbol": "TM",
      "private_sentinel": "must-not-be-echoed",
      "production_targets": []
    }
  ]
}
"#,
    )
    .expect("malformed module fixture must be written");

    let error = load_catalog_from_directory(directory.path()).unwrap_err();

    assert!(matches!(
        &error,
        CatalogLoadError::InvalidJson {
            module: factory_canvas::catalog_loader::CatalogModule::Buildables,
            kind: factory_canvas::catalog_loader::CatalogJsonErrorKind::Schema,
            ..
        }
    ));
    assert!(!error.to_string().contains("must-not-be-echoed"));
    assert!(!error
        .to_string()
        .contains(&directory.path().display().to_string()));
}

#[test]
fn symlinked_module_resolving_outside_the_package_root_is_rejected() {
    let directory = TestDirectory::with_public_package("symlink-escape");
    let outside = TestDirectory::new("symlink-target");
    let inside_module = directory.path().join("regions.json");
    let outside_module = outside.path().join("regions.json");
    fs::rename(&inside_module, &outside_module).expect("regions module must move outside root");

    if let Err(error) = create_file_symlink(&outside_module, &inside_module) {
        if error.kind() == std::io::ErrorKind::PermissionDenied
            || error.raw_os_error() == Some(1314)
        {
            return;
        }
        panic!("test symlink must be created: {error}");
    }

    let error = load_catalog_from_directory(directory.path()).unwrap_err();

    assert_eq!(
        error,
        CatalogLoadError::ModuleOutsideRoot {
            module: factory_canvas::catalog_loader::CatalogModule::Regions,
        }
    );
    assert!(!error
        .to_string()
        .contains(&outside.path().display().to_string()));
}
