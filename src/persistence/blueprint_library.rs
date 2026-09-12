//! Local, offline persistence for the player's saved blueprint library.
//!
//! This module composes three already-shipped building blocks: the
//! `Blueprint`/`BlueprintId` domain types (`crate::domain::blueprint`), the
//! blueprint document codec (`crate::persistence::blueprint_document`), and
//! the atomic same-directory write primitive
//! (`crate::persistence::atomic_file`). It adds no new domain type and no
//! UI — see `specs/001-blueprint-library/spec.md` FR-012.

use crate::domain::blueprint::{Blueprint, BlueprintId};
use crate::domain::catalog::Catalog;
use crate::persistence::atomic_file::{write_atomically, AtomicWriteError};
use crate::persistence::blueprint_document::{
    decode_blueprint_document, encode_blueprint_document, encode_blueprint_document_as,
    BlueprintDocumentError,
};
use crate::persistence::factory_document::CatalogCompatibility;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;
use time::OffsetDateTime;

/// Filename suffix appended to a blueprint's ID to form its on-disk
/// filename. See `specs/001-blueprint-library/data-model.md` "Filename
/// convention".
const BLUEPRINT_FILE_SUFFIX: &str = ".factory-blueprint.json";

/// Maximum number of `BlueprintId::generate()` retries `save` will attempt
/// before giving up when every generated ID collides with an existing
/// file. A UUIDv4 collision is astronomically unlikely; this bound exists
/// only so a pathological environment fails safely instead of looping
/// forever (FR-010; research.md §7).
const MAX_ID_COLLISION_RETRIES: u32 = 8;

/// A local, per-user library of saved blueprints backed by one JSON file
/// per blueprint under `root`.
///
/// Constructing a `BlueprintLibrary` performs no I/O: the root directory is
/// created lazily, only inside `save`, so that a fresh install or a
/// not-yet-used library never fails just from being constructed (FR-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintLibrary {
    root: PathBuf,
}

impl BlueprintLibrary {
    /// Creates a library rooted at an explicit directory. Tests always use
    /// this constructor with a temporary directory; production code should
    /// prefer [`BlueprintLibrary::default_for_user`].
    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    /// Creates a library rooted at the default per-user, per-application,
    /// OS-standard storage location: `%LOCALAPPDATA%/Factory Canvas/blueprints`.
    ///
    /// Returns `None` if `LOCALAPPDATA` is not set in the environment, which
    /// does not happen in practice on a real Windows user session but is
    /// surfaced rather than papered over with an arbitrary fallback.
    pub fn default_for_user() -> Option<Self> {
        let local_app_data = std::env::var_os("LOCALAPPDATA")?;
        let root = PathBuf::from(local_app_data)
            .join("Factory Canvas")
            .join("blueprints");
        Some(Self::at(root))
    }

    /// The directory this library instance reads from and writes to.
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Persists `blueprint` as a new file in this library.
    ///
    /// Creates the storage root automatically if it does not exist yet
    /// (FR-002). If `blueprint`'s ID collides with an ID already present in
    /// storage, a fresh ID is generated and the document is re-persisted
    /// under that new identity — not merely under a new filename — and
    /// retried up to a small fixed bound before giving up (FR-010): the
    /// caller-visible identity of the saved blueprint may therefore differ
    /// from `blueprint.id()` on return; callers that need the final ID
    /// should re-list the library. A failing or interrupted save never
    /// modifies any previously stored file (FR-011), inherited directly
    /// from
    /// [`write_atomically`](crate::persistence::atomic_file::write_atomically)'s
    /// same-directory temp-file-and-rename guarantee.
    pub fn save(&self, blueprint: &Blueprint) -> Result<(), BlueprintLibrarySaveError> {
        fs::create_dir_all(&self.root)
            .map_err(|error| BlueprintLibrarySaveError::Io { kind: error.kind() })?;

        let mut candidate_id = blueprint.id().clone();
        let mut encoded =
            encode_blueprint_document(blueprint).map_err(BlueprintLibrarySaveError::Encoding)?;
        let mut attempts_remaining = MAX_ID_COLLISION_RETRIES;
        loop {
            let path = self.path_for_id(&candidate_id);
            if !path.exists() {
                return write_atomically(&path, &encoded).map_err(|error: AtomicWriteError| {
                    BlueprintLibrarySaveError::Io { kind: error.kind() }
                });
            }
            if attempts_remaining == 0 {
                return Err(BlueprintLibrarySaveError::IdCollisionExhausted);
            }
            attempts_remaining -= 1;
            candidate_id = BlueprintId::generate();
            encoded = encode_blueprint_document_as(blueprint, &candidate_id)
                .map_err(BlueprintLibrarySaveError::Encoding)?;
        }
    }

    /// Lists every blueprint discoverable in this library's storage root.
    ///
    /// Never fails as a whole: a single unreadable or ambiguous file is
    /// isolated into `invalid_entries` rather than aborting the entire
    /// listing (FR-007, FR-009). If the storage root does not exist yet
    /// (nothing has been saved), returns an empty listing rather than an
    /// error.
    pub fn list(&self, active_catalog: &Catalog) -> BlueprintLibraryListing {
        let read_dir = match fs::read_dir(&self.root) {
            Ok(read_dir) => read_dir,
            Err(_) => return BlueprintLibraryListing::default(),
        };

        let mut invalid_entries = Vec::new();
        let mut candidates: Vec<DecodedCandidate> = Vec::new();

        for dir_entry in read_dir.flatten() {
            let Ok(file_type) = dir_entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                // FR-006: symlinks are ignored rather than followed.
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let file_name = dir_entry.file_name();
            if parse_blueprint_id_from_file_name(&file_name).is_none() {
                // FR-005: not a blueprint file at all, per the naming
                // convention. Silently ignored, not a warning.
                continue;
            }

            let bytes = match fs::read(dir_entry.path()) {
                Ok(bytes) => bytes,
                Err(_) => {
                    invalid_entries.push(InvalidLibraryEntry {
                        reason: InvalidLibraryEntryReason::UnreadableOrMalformed,
                    });
                    continue;
                }
            };
            match decode_blueprint_document(&bytes, active_catalog.clone()) {
                Ok(loaded) => {
                    let entry = BlueprintLibraryEntry {
                        id: loaded.blueprint.id().clone(),
                        name: loaded.blueprint.metadata().name().to_owned(),
                        node_count: loaded.blueprint.nodes().len(),
                        updated_at: loaded.blueprint.metadata().updated_at(),
                        compatibility: loaded.compatibility,
                        interface_names: loaded
                            .blueprint
                            .interfaces()
                            .iter()
                            .map(|interface| interface.name().to_owned())
                            .collect(),
                    };
                    candidates.push(DecodedCandidate { entry, file_name });
                }
                Err(_) => {
                    invalid_entries.push(InvalidLibraryEntry {
                        reason: InvalidLibraryEntryReason::UnreadableOrMalformed,
                    });
                }
            }
        }

        // Group by the blueprint identity each file's *content* declares
        // (FR-009) — independent of the eventual display order, since two
        // files sharing an identity could in principle carry different
        // names (e.g. one was hand-edited after copying). `BlueprintId`
        // implements neither `Hash` nor `Ord`, so grouping is done by
        // sorting on `id().as_str()` (a plain `&str`, which is `Ord`),
        // tiebreaking on name then filename only to pick a deterministic
        // winner within an identity group.
        candidates.sort_by(|left, right| {
            left.entry
                .id
                .as_str()
                .cmp(right.entry.id.as_str())
                .then_with(|| left.entry.name.cmp(&right.entry.name))
                .then_with(|| left.file_name.cmp(&right.file_name))
        });

        let mut entries = Vec::with_capacity(candidates.len());
        let mut index = 0;
        while index < candidates.len() {
            let mut group_end = index + 1;
            while group_end < candidates.len()
                && candidates[group_end].entry.id.as_str() == candidates[index].entry.id.as_str()
            {
                group_end += 1;
            }
            // FR-009: when two or more stored files claim the same
            // blueprint identity, exactly one becomes the valid entry (the
            // first per the tiebreak sort above) and every other one
            // becomes its own invalid-entry warning rather than a second,
            // ambiguous list entry.
            entries.push(candidates[index].entry.clone());
            for _ in &candidates[index + 1..group_end] {
                invalid_entries.push(InvalidLibraryEntry {
                    reason: InvalidLibraryEntryReason::DuplicateBlueprintId,
                });
            }
            index = group_end;
        }

        // FR-004: final display order is alphabetical by name, blueprint ID
        // as tiebreak — independent of the identity-grouping order above.
        entries.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.as_str().cmp(right.id.as_str()))
        });

        BlueprintLibraryListing {
            entries,
            invalid_entries,
        }
    }

    fn path_for_id(&self, id: &BlueprintId) -> PathBuf {
        self.root
            .join(format!("{}{BLUEPRINT_FILE_SUFFIX}", id.as_str()))
    }

    /// Loads the single, complete `Blueprint` stored under `id` — every
    /// node and interface, not just the summary `list()` returns.
    ///
    /// **Deviation from the original plan, discovered during
    /// implementation**: research.md's Decision 2 states insertion "reads
    /// an already-loaded `Blueprint`... it does not add a new library
    /// operation," on the assumption the UI already held a full
    /// `Blueprint` value once a library entry was chosen. In practice,
    /// `list()` only ever returns `BlueprintLibraryEntry` (id, name, node
    /// count, timestamp, compatibility — deliberately not the full value,
    /// per this file's own doc comment on that type) — actually inserting
    /// a chosen entry's blueprint requires reading its complete node data
    /// from disk first. This method exists to make that reachable; it
    /// reuses the exact same `decode_blueprint_document` call `list()`
    /// already makes per candidate file, just for one already-known ID
    /// instead of every file in the root, and introduces no new codec
    /// path or storage format.
    pub fn load(
        &self,
        id: &BlueprintId,
        active_catalog: &Catalog,
    ) -> Result<Blueprint, BlueprintLibraryLoadError> {
        let path = self.path_for_id(id);
        let bytes =
            fs::read(&path).map_err(|_| BlueprintLibraryLoadError::UnreadableOrMalformed)?;
        let loaded = decode_blueprint_document(&bytes, active_catalog.clone())
            .map_err(|_| BlueprintLibraryLoadError::UnreadableOrMalformed)?;
        Ok(loaded.blueprint)
    }
}

/// One successfully decoded file, paired with its filename for use only as
/// an internal tiebreak while grouping duplicate identities in `list`. The
/// filename is never exposed on [`BlueprintLibraryEntry`] itself.
struct DecodedCandidate {
    entry: BlueprintLibraryEntry,
    file_name: OsString,
}

/// Parses a file name against the blueprint-file naming convention
/// (`specs/001-blueprint-library/data-model.md` "Filename convention"):
/// `<blueprint_id>.factory-blueprint.json`, where `<blueprint_id>` must
/// itself round-trip through [`BlueprintId::parse`]. Used to decide whether
/// a directory entry is a blueprint file at all (FR-005).
fn parse_blueprint_id_from_file_name(file_name: &OsStr) -> Option<BlueprintId> {
    let file_name = file_name.to_str()?;
    let id_part = file_name.strip_suffix(BLUEPRINT_FILE_SUFFIX)?;
    BlueprintId::parse(id_part).ok()
}

/// The outcome of one [`BlueprintLibrary::list`] call.
///
/// Deliberately two separate, already-sorted collections rather than one
/// mixed list, so a UI consumer never has to filter by variant itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlueprintLibraryListing {
    /// Valid blueprints, sorted alphabetically by name ascending, then by
    /// blueprint ID ascending as a tiebreak (FR-004).
    pub entries: Vec<BlueprintLibraryEntry>,
    /// One entry per file that could not become a valid, unambiguous list
    /// entry — both genuinely malformed files and every loser of a
    /// duplicate-ID conflict (FR-009).
    pub invalid_entries: Vec<InvalidLibraryEntry>,
}

/// The summary shown per blueprint in the library listing (spec.md
/// "Blueprint Library Entry"). Deliberately not the full [`Blueprint`]
/// value, so listing never requires holding every node of every blueprint
/// in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintLibraryEntry {
    id: BlueprintId,
    name: String,
    node_count: usize,
    updated_at: OffsetDateTime,
    compatibility: CatalogCompatibility,
    interface_names: Vec<String>,
}

impl BlueprintLibraryEntry {
    pub fn id(&self) -> &BlueprintId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    pub const fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }

    pub const fn compatibility(&self) -> CatalogCompatibility {
        self.compatibility
    }

    /// This blueprint's named interfaces (FR-010), purely descriptive — no
    /// connection or flow state is implied by their presence here.
    pub fn interface_names(&self) -> &[String] {
        &self.interface_names
    }
}

/// A safe, non-leaking notice associated with one stored file that could
/// not be read as a valid, unambiguous blueprint (spec.md "Invalid Entry
/// Warning").
///
/// Structurally carries no path, no raw bytes, and no technical identifier
/// — not merely omitted from `Display`, but absent from the type itself —
/// so FR-008 holds by construction rather than by discipline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidLibraryEntry {
    reason: InvalidLibraryEntryReason,
}

impl InvalidLibraryEntry {
    pub const fn reason(&self) -> InvalidLibraryEntryReason {
        self.reason
    }
}

/// Why one stored file became an [`InvalidLibraryEntry`] instead of a
/// [`BlueprintLibraryEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidLibraryEntryReason {
    /// The file could not be decoded as a valid blueprint document at all.
    /// Every [`BlueprintDocumentError`] case is collapsed to this one safe
    /// reason — the specific decode failure is exactly the kind of
    /// technical detail FR-008 forbids surfacing.
    UnreadableOrMalformed,
    /// The file decoded successfully but lost the deterministic tiebreak
    /// against another file claiming the same [`BlueprintId`].
    DuplicateBlueprintId,
}

/// Why [`BlueprintLibrary::save`] failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintLibrarySaveError {
    /// The underlying atomic write failed (directory not creatable/writable,
    /// disk full, permissions, etc.).
    Io { kind: io::ErrorKind },
    /// Encoding the document failed. Unreachable in practice for a valid
    /// in-memory [`Blueprint`], kept only for exhaustiveness with the
    /// codec's own error contract.
    Encoding(BlueprintDocumentError),
    /// The bounded collision-retry loop could not find a free ID.
    IdCollisionExhausted,
}

/// Why [`BlueprintLibrary::load`] failed.
///
/// Collapses every possible cause (file missing since the listing was
/// cached, I/O failure, decode failure) into one safe reason — same
/// privacy discipline as [`InvalidLibraryEntryReason::UnreadableOrMalformed`],
/// since a caller only needs to know insertion cannot proceed, not why in
/// technical terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintLibraryLoadError {
    UnreadableOrMalformed,
}
