use factory_canvas::domain::blueprint::{
    Blueprint, BlueprintEntityId, BlueprintId, BlueprintNodeInput,
};
use factory_canvas::domain::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, CatalogId, CatalogMetadata,
    CategoryId, RegionDefinition, RegionId,
};
use factory_canvas::domain::document::DocumentMetadata;
use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
use factory_canvas::persistence::blueprint_library::{
    BlueprintLibrary, BlueprintLibraryEntry, BlueprintLibrarySaveError,
};
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

fn test_catalog() -> Catalog {
    let region_id = RegionId::new("test_region").unwrap();
    let base_id = BaseId::new("test_base").unwrap();
    Catalog::new(
        CatalogMetadata::new(
            CatalogId::new("test_catalog").unwrap(),
            Version::new(1, 0, 0),
            "Test Catalog",
        ),
        base_id.clone(),
        vec![RegionDefinition::new(region_id.clone(), "Test Region")],
        vec![BaseDefinition::new(
            base_id,
            "Test Base",
            region_id,
            GridSize::new(30, 30).unwrap(),
        )],
        vec![BuildableDefinition::new(
            BuildableId::new("test_machine").unwrap(),
            "Test Machine",
            CategoryId::new("test_category").unwrap(),
            "TM",
            GridSize::new(2, 3).unwrap(),
            Vec::new(),
            None,
        )],
        Vec::new(),
    )
    .unwrap()
}

fn fixed_timestamp() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap()
}

/// A blueprint ID built from a small integer, purely so tests can compare
/// two IDs' expected lexical order without depending on `generate()`'s
/// randomness. `{:032x}` always yields exactly the 32 lowercase hex
/// characters `BlueprintId::parse` requires.
fn fixed_id(value: u32) -> BlueprintId {
    BlueprintId::parse(&format!("blueprint_{value:032x}")).unwrap()
}

fn blueprint_with_id_and_name(catalog: &Catalog, id: BlueprintId, name: &str) -> Blueprint {
    let metadata = DocumentMetadata::new(name, None, fixed_timestamp(), fixed_timestamp()).unwrap();
    let nodes = vec![BlueprintNodeInput {
        id: BlueprintEntityId::new(1),
        buildable_id: BuildableId::new("test_machine").unwrap(),
        relative_origin: GridPoint::new(0, 0),
        rotation: Rotation::Zero,
        production_target: None,
    }];
    Blueprint::from_nodes(id, catalog.clone(), metadata, nodes, Vec::new()).unwrap()
}

fn blueprint_named(catalog: &Catalog, name: &str) -> Blueprint {
    blueprint_with_id_and_name(catalog, BlueprintId::generate(), name)
}

/// Reads the single file expected to exist directly inside `root` (not
/// recursive), asserting there really is exactly one. Used by tests that
/// need to inspect or tamper with a file the library itself just wrote,
/// without depending on the library's private filename convention.
fn only_file_in(root: &Path) -> (PathBuf, Vec<u8>) {
    let files: Vec<PathBuf> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(
        files.len(),
        1,
        "expected exactly one file in {}, found {files:?}",
        root.display()
    );
    let path = files[0].clone();
    let bytes = fs::read(&path).unwrap();
    (path, bytes)
}

// ---------------------------------------------------------------------
// User Story 1 (P1): Blueprints survive a restart
// ---------------------------------------------------------------------

#[test]
fn saved_blueprint_is_discovered_by_a_new_library_instance() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();

    {
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let blueprint = blueprint_named(&catalog, "Alpha");
        library.save(&blueprint).unwrap();
    } // `library` dropped here, simulating the application closing.

    let restarted_library = BlueprintLibrary::at(directory.path().to_path_buf());
    let listing = restarted_library.list(&catalog);

    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].name(), "Alpha");
    assert_eq!(listing.entries[0].node_count(), 1);
    assert_eq!(listing.entries[0].updated_at(), fixed_timestamp());
    assert!(listing.invalid_entries.is_empty());
}

#[test]
fn first_save_creates_the_storage_root_automatically() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("blueprints");
    assert!(!root.exists(), "the root must not pre-exist for this test");

    let catalog = test_catalog();
    let library = BlueprintLibrary::at(root.clone());
    let blueprint = blueprint_named(&catalog, "First Run");

    let result = library.save(&blueprint);

    assert_eq!(result, Ok(()));
    assert!(root.is_dir(), "save must create the storage root");

    let listing = library.list(&catalog);
    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].name(), "First Run");
}

/// Guards a directory with a `deny write` ACL for the current user via
/// `icacls`, restoring it on drop so a mid-test panic never leaves a
/// permanently locked-down temp directory behind (which would also break
/// `tempfile::TempDir`'s own cleanup). Windows-only: `icacls` has no
/// equivalent on other platforms, consistent with this product's
/// Windows-only target (`docs/adr/0001-editor-ui.md`).
#[cfg(windows)]
struct DenyWriteGuard {
    path: PathBuf,
    username: String,
}

#[cfg(windows)]
impl DenyWriteGuard {
    fn apply(path: &Path) -> Self {
        let username = std::env::var("USERNAME").expect("USERNAME must be set on Windows");
        let status = std::process::Command::new("icacls")
            .arg(path)
            .arg("/deny")
            .arg(format!("{username}:(W,WD,AD)"))
            .status()
            .expect("icacls must be runnable on Windows");
        assert!(status.success(), "icacls /deny must succeed");
        Self {
            path: path.to_path_buf(),
            username,
        }
    }
}

#[cfg(windows)]
impl Drop for DenyWriteGuard {
    fn drop(&mut self) {
        // Best-effort: a failure here must not mask the test's real
        // assertion failure (if any) with a panic-during-unwind abort.
        let _ = std::process::Command::new("icacls")
            .arg(&self.path)
            .arg("/remove:d")
            .arg(&self.username)
            .status();
    }
}

#[test]
#[cfg(windows)]
fn interrupted_or_failing_save_leaves_previously_stored_blueprint_untouched() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    let first = blueprint_named(&catalog, "First");
    library.save(&first).unwrap();
    let (first_path, first_bytes_before) = only_file_in(directory.path());

    {
        let _guard = DenyWriteGuard::apply(directory.path());

        let second = blueprint_named(&catalog, "Second");
        let result = library.save(&second);

        assert!(
            matches!(result, Err(BlueprintLibrarySaveError::Io { .. })),
            "save into a write-denied directory must fail with Io, got {result:?}"
        );
    } // Guard dropped here: permissions restored even if an assertion above panicked.

    let first_bytes_after = fs::read(&first_path).unwrap();
    assert_eq!(
        first_bytes_after, first_bytes_before,
        "a failed save must not modify the previously stored file"
    );

    let listing = library.list(&catalog);
    assert_eq!(listing.entries.len(), 1);
    assert_eq!(listing.entries[0].name(), "First");
    assert!(listing.invalid_entries.is_empty());
}

// ---------------------------------------------------------------------
// User Story 2 (P2): One broken file never hides the rest of the library
// ---------------------------------------------------------------------

#[test]
fn invalid_file_is_isolated_while_valid_blueprints_remain_listed() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    for name in ["Alpha", "Bravo", "Charlie"] {
        library.save(&blueprint_named(&catalog, name)).unwrap();
    }
    fs::write(
        directory
            .path()
            .join("blueprint_deadbeefdeadbeefdeadbeefdeadbeef.factory-blueprint.json"),
        b"this is not valid blueprint document json",
    )
    .unwrap();

    let listing = library.list(&catalog);

    assert_eq!(
        listing.entries.len(),
        3,
        "all valid blueprints must remain listed"
    );
    assert_eq!(
        listing.invalid_entries.len(),
        1,
        "exactly one warning for the one malformed file"
    );
    assert_eq!(
        listing.invalid_entries[0].reason(),
        factory_canvas::persistence::blueprint_library::InvalidLibraryEntryReason::UnreadableOrMalformed
    );
}

#[test]
fn invalid_entry_warning_never_leaks_path_content_or_identifier() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    library.save(&blueprint_named(&catalog, "Alpha")).unwrap();
    let secret_marker = "TOP_SECRET_RAW_BYTES_MARKER";
    fs::write(
        directory
            .path()
            .join("blueprint_deadbeefdeadbeefdeadbeefdeadbeef.factory-blueprint.json"),
        secret_marker.as_bytes(),
    )
    .unwrap();

    let listing = library.list(&catalog);
    assert_eq!(listing.invalid_entries.len(), 1);

    let debug_output = format!("{:?}", listing.invalid_entries);
    let directory_path_string = directory.path().to_string_lossy().to_string();

    assert!(
        !debug_output.contains(&directory_path_string),
        "warning must not leak the storage directory's path"
    );
    assert!(
        !debug_output.contains(secret_marker),
        "warning must not leak the malformed file's raw content"
    );
    assert!(
        !debug_output.contains("deadbeef"),
        "warning must not leak the file's technical identifier"
    );
}

// ---------------------------------------------------------------------
// User Story 3 (P3): Unusual storage contents behave predictably
// ---------------------------------------------------------------------

#[test]
fn unrelated_file_in_storage_is_silently_ignored() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    library.save(&blueprint_named(&catalog, "Alpha")).unwrap();
    fs::write(directory.path().join("notes.txt"), b"just some notes").unwrap();

    let listing = library.list(&catalog);

    assert_eq!(listing.entries.len(), 1);
    assert!(listing.invalid_entries.is_empty());
}

#[test]
#[cfg(windows)]
fn symlinked_blueprint_file_is_not_followed_or_listed() {
    let library_directory = tempfile::tempdir().unwrap();
    let elsewhere_directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();

    let library = BlueprintLibrary::at(library_directory.path().to_path_buf());
    library
        .save(&blueprint_named(&catalog, "OnlyDirect"))
        .unwrap();

    let elsewhere_library = BlueprintLibrary::at(elsewhere_directory.path().to_path_buf());
    elsewhere_library
        .save(&blueprint_named(&catalog, "ShouldNotAppear"))
        .unwrap();
    let (elsewhere_file, _) = only_file_in(elsewhere_directory.path());

    let link_path = library_directory
        .path()
        .join("blueprint_cafecafecafecafecafecafecafecafe.factory-blueprint.json");
    if let Err(error) = std::os::windows::fs::symlink_file(&elsewhere_file, &link_path) {
        if error.raw_os_error() == Some(1314) {
            // ERROR_PRIVILEGE_NOT_HELD: this Windows session has neither
            // Developer Mode nor elevation, so it cannot create symbolic
            // links at all. The behavior this test verifies (a real
            // symlink is skipped, not followed) cannot be exercised
            // without one; skip rather than fail the whole suite on an
            // environment limitation unrelated to the code under test.
            eprintln!(
                "skipping symlinked_blueprint_file_is_not_followed_or_listed: \
                 this Windows session cannot create symlinks (enable Developer \
                 Mode or run elevated to exercise this test)"
            );
            return;
        }
        panic!("unexpected error creating test symlink: {error}");
    }

    let listing = library.list(&catalog);

    assert_eq!(
        listing.entries.len(),
        1,
        "only the directly-saved blueprint may appear, the symlink must be ignored"
    );
    assert_eq!(listing.entries[0].name(), "OnlyDirect");
    assert!(listing.invalid_entries.is_empty());
}

#[test]
fn duplicate_blueprint_identity_resolves_to_one_entry_and_one_warning() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    let shared_id = fixed_id(1);
    let blueprint = blueprint_with_id_and_name(&catalog, shared_id.clone(), "Duplicated");
    library.save(&blueprint).unwrap();
    let (_, bytes) = only_file_in(directory.path());

    // Simulate a manual file copy: same decoded identity, different
    // filename, bypassing the library's own save() API entirely.
    let copied_path = directory
        .path()
        .join("blueprint_00000000000000000000000000000099.factory-blueprint.json");
    fs::write(&copied_path, &bytes).unwrap();

    let listing = library.list(&catalog);

    assert_eq!(
        listing.entries.len(),
        1,
        "two files claiming the same identity must never appear as two entries"
    );
    assert_eq!(listing.entries[0].id(), &shared_id);
    assert_eq!(listing.invalid_entries.len(), 1);
    assert_eq!(
        listing.invalid_entries[0].reason(),
        factory_canvas::persistence::blueprint_library::InvalidLibraryEntryReason::DuplicateBlueprintId
    );
}

#[test]
fn saving_a_colliding_blueprint_id_retries_with_a_new_identifier() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    let shared_id = fixed_id(7);
    let first = blueprint_with_id_and_name(&catalog, shared_id.clone(), "First");
    library.save(&first).unwrap();

    let second = blueprint_with_id_and_name(&catalog, shared_id, "Second");
    let result = library.save(&second);

    assert_eq!(
        result,
        Ok(()),
        "saving a colliding, unrelated blueprint must still succeed"
    );

    let listing = library.list(&catalog);
    assert_eq!(
        listing.entries.len(),
        2,
        "the collision must be avoided rather than overwriting the first blueprint"
    );
    let mut ids: Vec<&str> = listing
        .entries
        .iter()
        .map(|entry| entry.id().as_str())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(
        ids.len(),
        2,
        "the two entries must have distinct identifiers"
    );
    assert!(listing.invalid_entries.is_empty());
}

// ---------------------------------------------------------------------
// Cross-cutting: FR-004 / SC-004 ordering and determinism
//
// Added beyond tasks.md's original T001-T018 in response to
// /speckit-analyze finding C1: FR-004 (and its measurable counterpart
// SC-004) had an implementation task but no test verifying the sort order
// the /speckit-clarify session established, or that repeated listings are
// stable.
// ---------------------------------------------------------------------

#[test]
fn listing_orders_entries_alphabetically_by_name_with_id_tiebreak_and_is_stable_across_repeated_calls(
) {
    let directory = tempfile::tempdir().unwrap();
    let catalog = test_catalog();
    let library = BlueprintLibrary::at(directory.path().to_path_buf());

    // Inserted out of alphabetical order on purpose.
    library.save(&blueprint_named(&catalog, "Charlie")).unwrap();
    library.save(&blueprint_named(&catalog, "Alpha")).unwrap();
    library.save(&blueprint_named(&catalog, "Bravo")).unwrap();
    // Two blueprints sharing one name, with known, orderable IDs, to
    // exercise the ID tiebreak specifically.
    let lower_id = fixed_id(1);
    let higher_id = fixed_id(2);
    library
        .save(&blueprint_with_id_and_name(
            &catalog,
            higher_id.clone(),
            "Same",
        ))
        .unwrap();
    library
        .save(&blueprint_with_id_and_name(
            &catalog,
            lower_id.clone(),
            "Same",
        ))
        .unwrap();

    let first_listing = library.list(&catalog);
    let second_listing = library.list(&catalog);

    let names: Vec<&str> = first_listing
        .entries
        .iter()
        .map(BlueprintLibraryEntry::name)
        .collect();
    assert_eq!(names, vec!["Alpha", "Bravo", "Charlie", "Same", "Same"]);
    assert_eq!(
        first_listing.entries[3].id(),
        &lower_id,
        "lower ID must sort first among equal names"
    );
    assert_eq!(first_listing.entries[4].id(), &higher_id);
    assert!(first_listing.invalid_entries.is_empty());

    assert_eq!(
        first_listing, second_listing,
        "listing the same, unchanged storage contents twice must return identical order both times"
    );
}
