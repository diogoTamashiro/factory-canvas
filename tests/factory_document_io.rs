use factory_canvas::catalog_loader::load_embedded_public_catalog;
use factory_canvas::domain::document::DocumentMetadata;
use factory_canvas::domain::layout::FactoryLayout;
use factory_canvas::persistence::factory_document::{
    encode_factory_document, load_factory_document, save_factory_document, CatalogCompatibility,
    FactoryDocumentError, FactoryDocumentIoOperation,
};
use std::fs;
use std::io::ErrorKind;
use time::OffsetDateTime;

fn empty_factory() -> (FactoryLayout, DocumentMetadata) {
    let catalog = load_embedded_public_catalog().expect("public catalog must load");
    let layout = FactoryLayout::new(catalog.clone(), catalog.default_base().id().clone())
        .expect("default base must produce a layout");
    let timestamp =
        OffsetDateTime::from_unix_timestamp(1_700_000_000).expect("test timestamp must be valid");
    let metadata = DocumentMetadata::new("I/O Test", None, timestamp, timestamp)
        .expect("test metadata must be valid");
    (layout, metadata)
}

#[test]
fn saving_factory_creates_complete_pretty_json_in_target_directory() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("factory.json");
    let (layout, metadata) = empty_factory();
    let expected =
        encode_factory_document(&layout, Some(1), &metadata).expect("test factory must encode");

    save_factory_document(&target, &layout, Some(1), &metadata).expect("test factory must save");

    let actual = fs::read(&target).expect("saved factory must be readable");
    assert_eq!(actual, expected);
    assert!(actual.ends_with(b"\n"));
    assert!(actual.windows(3).any(|window| window == b"\n  "));
    serde_json::from_slice::<serde_json::Value>(&actual)
        .expect("saved factory must contain complete JSON");

    let entries = fs::read_dir(directory.path())
        .expect("test directory must be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("test directory entries must be readable");
    assert_eq!(entries.len(), 1, "temporary file must not remain");
    assert_eq!(entries[0].path(), target);
}

#[test]
fn saving_factory_atomically_replaces_existing_complete_file() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("factory.json");
    fs::write(&target, b"previous complete document\n").expect("previous target must be created");
    let (layout, metadata) = empty_factory();
    let expected =
        encode_factory_document(&layout, Some(1), &metadata).expect("test factory must encode");

    save_factory_document(&target, &layout, Some(1), &metadata).expect("test factory must save");

    assert_eq!(
        fs::read(&target).expect("replaced target must be readable"),
        expected
    );
    let entries = fs::read_dir(directory.path())
        .expect("test directory must be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("test directory entries must be readable");
    assert_eq!(entries.len(), 1, "temporary file must not remain");
    assert_eq!(entries[0].path(), target);
}

#[test]
fn loading_factory_reads_and_validates_the_complete_file() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("factory.json");
    let (layout, metadata) = empty_factory();
    let catalog = layout.catalog().clone();
    save_factory_document(&target, &layout, Some(1), &metadata).expect("test factory must save");

    let loaded =
        load_factory_document(&target, catalog).expect("saved factory must load from disk");

    assert_eq!(loaded.layout, layout);
    assert_eq!(loaded.next_entity_id, Some(1));
    assert_eq!(loaded.metadata, metadata);
    assert_eq!(loaded.compatibility, CatalogCompatibility::Exact);
}

#[test]
fn saving_factory_to_missing_parent_returns_safe_create_error() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let missing_parent = directory.path().join("private-parent-sentinel");
    let target = missing_parent.join("private-factory-sentinel.json");
    let (layout, metadata) = empty_factory();

    let error = save_factory_document(&target, &layout, Some(1), &metadata)
        .expect_err("missing parent must reject the save");

    assert_eq!(
        error,
        FactoryDocumentError::Io {
            operation: FactoryDocumentIoOperation::CreateTemporary,
            kind: ErrorKind::NotFound,
        }
    );
    assert!(!missing_parent.exists());
    let rendered = format!("{error:?}\n{error}");
    assert!(!rendered.contains("private-parent-sentinel"));
    assert!(!rendered.contains("private-factory-sentinel"));
    assert!(!rendered.contains(&target.display().to_string()));
}

#[test]
fn saving_factory_over_directory_returns_safe_replace_error_and_preserves_target() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("private-directory-sentinel");
    fs::create_dir(&target).expect("target directory must be created");
    let (layout, metadata) = empty_factory();

    let error = save_factory_document(&target, &layout, Some(1), &metadata)
        .expect_err("directory target must reject replacement");

    assert!(matches!(
        error,
        FactoryDocumentError::Io {
            operation: FactoryDocumentIoOperation::ReplaceTarget,
            ..
        }
    ));
    assert!(target.is_dir(), "existing directory must remain intact");
    let entries = fs::read_dir(directory.path())
        .expect("test directory must be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("test directory entries must be readable");
    assert_eq!(entries.len(), 1, "temporary file must be cleaned up");
    assert_eq!(entries[0].path(), target);
    let rendered = format!("{error:?}\n{error}");
    assert!(!rendered.contains("private-directory-sentinel"));
    assert!(!rendered.contains(&target.display().to_string()));
}

#[test]
fn loading_missing_factory_returns_safe_read_error() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("private-missing-sentinel.json");
    let catalog = load_embedded_public_catalog().expect("public catalog must load");

    let error = load_factory_document(&target, catalog)
        .expect_err("missing factory document must reject the load");

    assert_eq!(
        error,
        FactoryDocumentError::Io {
            operation: FactoryDocumentIoOperation::Read,
            kind: ErrorKind::NotFound,
        }
    );
    let rendered = format!("{error:?}\n{error}");
    assert!(!rendered.contains("private-missing-sentinel"));
    assert!(!rendered.contains(&target.display().to_string()));
}

#[test]
fn loading_invalid_factory_returns_codec_error_without_path_or_bytes() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("private-invalid-path-sentinel.json");
    let private_bytes = br#"{
        "schema_version": 1,
        "private-content-sentinel": ]
    }"#;
    fs::write(&target, private_bytes).expect("invalid test document must be written");
    let catalog = load_embedded_public_catalog().expect("public catalog must load");

    let error = load_factory_document(&target, catalog)
        .expect_err("invalid factory document must reject the load");

    assert!(matches!(error, FactoryDocumentError::InvalidJson { .. }));
    let rendered = format!("{error:?}\n{error}");
    assert!(!rendered.contains("private-content-sentinel"));
    assert!(!rendered.contains("private-invalid-path-sentinel"));
    assert!(!rendered.contains(&target.display().to_string()));
}

#[test]
fn invalid_factory_is_rejected_before_existing_target_is_touched() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let target = directory.path().join("factory.json");
    let previous = b"previous complete document\n";
    fs::write(&target, previous).expect("previous target must be created");
    let (layout, metadata) = empty_factory();

    let error = save_factory_document(&target, &layout, Some(0), &metadata)
        .expect_err("invalid allocator must reject the save");

    assert_eq!(error, FactoryDocumentError::InvalidNextEntityId);
    assert_eq!(
        fs::read(&target).expect("previous target must remain readable"),
        previous
    );
    let entries = fs::read_dir(directory.path())
        .expect("test directory must be readable")
        .collect::<Result<Vec<_>, _>>()
        .expect("test directory entries must be readable");
    assert_eq!(entries.len(), 1, "temporary file must not be created");
    assert_eq!(entries[0].path(), target);
}

#[test]
fn invalid_factory_validation_precedes_missing_parent_io() {
    let directory = tempfile::tempdir().expect("test directory must be created");
    let missing_parent = directory.path().join("missing-parent");
    let target = missing_parent.join("factory.json");
    let (layout, metadata) = empty_factory();

    let error = save_factory_document(&target, &layout, Some(0), &metadata)
        .expect_err("invalid allocator must win before filesystem access");

    assert_eq!(error, FactoryDocumentError::InvalidNextEntityId);
    assert!(!missing_parent.exists());
    assert!(
        fs::read_dir(directory.path())
            .expect("test directory must be readable")
            .next()
            .is_none(),
        "validation must complete before a temporary file is attempted"
    );
}
