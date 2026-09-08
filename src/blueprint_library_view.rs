//! View state for the blueprint library UI (Commit 8, Phase 4).
//!
//! Owns the app-facing bridge between `FactoryCanvasApp` and the two
//! already-shipped capabilities this feature builds on: `Blueprint`
//! capture (`crate::domain::blueprint::Blueprint::from_selection`, Commit
//! 5) and `BlueprintLibrary` persistence
//! (`factory_canvas::persistence::blueprint_library`, Commit 7). Adds no
//! new domain type and no new persistence mechanism — see
//! `specs/002-blueprint-library-ui/data-model.md`.
//!
//! `BlueprintLibraryView` performs **no I/O** except through
//! `connect_to_default_storage` (called exactly once, only from the real
//! production entry point) and `confirm_save`/`refresh` (only once a real,
//! explicitly-provided `BlueprintLibrary` is present). This mirrors
//! `BlueprintLibrary` itself, which already separates a no-I/O `at()`
//! constructor from the filesystem-touching `default_for_user()` — applied
//! one level up so that `FactoryCanvasApp::from_startup_catalog`, shared by
//! both production and every unit test, never touches the real user
//! profile just by constructing an app (see this file's "Design
//! correction" note in `data-model.md`).

use factory_canvas::domain::blueprint::{Blueprint, BlueprintId};
use factory_canvas::domain::catalog::Catalog;
use factory_canvas::domain::document::DocumentMetadata;
use factory_canvas::domain::layout::{EntityId, FactoryLayout};
use factory_canvas::persistence::blueprint_library::{
    BlueprintLibrary, BlueprintLibraryListing, BlueprintLibrarySaveError,
};
use time::OffsetDateTime;

/// The save-as-blueprint dialog's transient input state. Exists only while
/// the dialog is open; created whole by `begin_save` and destroyed whole by
/// `cancel_save` or by a `confirm_save` that actually attempts capture/save
/// (data-model.md: no partial/multi-step wizard state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingBlueprintSave {
    /// Frozen at dialog-open time, mirroring
    /// `FactoryCanvasApp::pending_instance_removal`/`pending_base_change`'s
    /// existing `Option<...>` idiom for every other pending-confirmation
    /// state in this app (research.md Decision 6).
    pub(crate) selected_ids: Vec<EntityId>,
    /// Bound directly to the modal's text field. Trimmed only at
    /// confirm/validation time, never on every keystroke.
    pub(crate) name_input: String,
}

/// App-internal view state for the blueprint library: the cached listing
/// shown in the sidebar, the library handle it was read from (if any), and
/// the save dialog's transient state while open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlueprintLibraryView {
    library: Option<BlueprintLibrary>,
    listing: BlueprintLibraryListing,
    pending_save: Option<PendingBlueprintSave>,
}

impl BlueprintLibraryView {
    /// Fully disconnected, no-I/O state: no library, an empty listing, no
    /// pending save. This is what `FactoryCanvasApp::from_startup_catalog`
    /// uses, so `Default` and every existing/new test remain exactly as
    /// hermetic as they are today.
    pub(crate) fn new() -> Self {
        Self {
            library: None,
            listing: BlueprintLibraryListing::default(),
            pending_save: None,
        }
    }

    /// Wraps an already-constructed `BlueprintLibrary` with an empty cached
    /// listing and no pending save. Performs no I/O itself (no `list`
    /// call) — tests use this with `BlueprintLibrary::at(temp_dir)`
    /// (mirroring how Commit 7's own tests construct `BlueprintLibrary`)
    /// when they need a real, isolated, temp-dir-backed library rather than
    /// the fully disconnected `new()` state.
    pub(crate) fn with_library(library: BlueprintLibrary) -> Self {
        Self {
            library: Some(library),
            listing: BlueprintLibraryListing::default(),
            pending_save: None,
        }
    }

    /// The **only** call site of `BlueprintLibrary::default_for_user()` in
    /// this feature. On success, replaces `self` with a freshly connected
    /// view and performs one `refresh` (research.md Decision 2's "one
    /// construction-time refresh", now explicit). On failure (Decision 9 —
    /// `LOCALAPPDATA` unset), leaves `self` in its current
    /// disconnected/degraded state rather than panicking. Called exactly
    /// once, only from the real `FactoryCanvasApp::new(creation_context)`
    /// production entry point — never from `from_startup_catalog`,
    /// `Default`, or any test.
    pub(crate) fn connect_to_default_storage(&mut self, catalog: &Catalog) {
        if let Some(library) = BlueprintLibrary::default_for_user() {
            *self = Self::with_library(library);
            self.refresh(catalog);
        }
    }

    /// Overwrites the cached listing with a fresh `library.list(catalog)`
    /// call. No-op if there is no connected library. Never called per
    /// frame (research.md Decision 2) — only at connection time and
    /// immediately after a successful save.
    pub(crate) fn refresh(&mut self, catalog: &Catalog) {
        if let Some(library) = &self.library {
            self.listing = library.list(catalog);
        }
    }

    /// Whether production storage is connected. A disconnected view is the
    /// explicit degraded state from research.md Decision 9; callers use
    /// this to disable the save action and render the persistent safe
    /// "unavailable" indication instead of silently doing nothing.
    pub(crate) fn is_connected(&self) -> bool {
        self.library.is_some()
    }

    /// The cached listing shown in the sidebar.
    pub(crate) fn listing(&self) -> &BlueprintLibraryListing {
        &self.listing
    }

    /// Whether a save dialog is currently open — used to gate other canvas
    /// edits while it is (spec Assumption #3), matching how
    /// `pending_base_change`/`pending_instance_removal` already gate
    /// `destructive_modal_open`.
    pub(crate) fn has_pending_save(&self) -> bool {
        self.pending_save.is_some()
    }

    /// The open dialog's transient state, if any.
    pub(crate) fn pending_save(&self) -> Option<&PendingBlueprintSave> {
        self.pending_save.as_ref()
    }

    /// Mutable access to the open dialog's name field, for the text-edit
    /// widget to bind to. `None` if no dialog is open.
    pub(crate) fn pending_save_name_mut(&mut self) -> Option<&mut String> {
        self.pending_save
            .as_mut()
            .map(|pending| &mut pending.name_input)
    }

    /// Opens the save dialog, freezing `selected_ids` as the exact set of
    /// canvas instances that will become the blueprint's nodes on confirm.
    /// Does not touch `listing`.
    pub(crate) fn begin_save(&mut self, selected_ids: Vec<EntityId>) {
        self.pending_save = Some(PendingBlueprintSave {
            selected_ids,
            name_input: String::new(),
        });
    }

    /// Closes the save dialog without creating anything. No side effect on
    /// `listing` or storage (spec FR-005).
    pub(crate) fn cancel_save(&mut self) {
        self.pending_save = None;
    }

    /// Attempts to confirm the currently open save dialog.
    ///
    /// Returns `None` if no dialog is open, no library is connected, or the
    /// trimmed name is blank. All three branches are no-ops: the caller is
    /// responsible for preventing an unavailable-library attempt (research.md
    /// Decision 9), while a blank name deliberately keeps the dialog open so
    /// the player can correct it.
    ///
    /// If the frozen selection no longer resolves (unreachable in normal UI
    /// flow because the modal blocks canvas edits), the dialog closes because
    /// that original selection cannot be recovered.
    ///
    /// On `Ok`, the dialog is closed and the listing is refreshed
    /// immediately (research.md Decision 2), so the new entry is visible
    /// without any further manual step. On `Err`, the dialog is also
    /// closed, but the listing is left untouched — a failed save must
    /// never fabricate a change to what is actually stored (FR-011).
    pub(crate) fn confirm_save(
        &mut self,
        layout: &FactoryLayout,
        catalog: &Catalog,
        now: OffsetDateTime,
    ) -> Option<Result<(), BlueprintLibrarySaveError>> {
        let pending = self.pending_save.as_ref()?;
        let library = self.library.as_ref()?;

        let trimmed_name = pending.name_input.trim();
        if trimmed_name.is_empty() {
            // This can be fixed by the player typing a valid name — keep
            // the dialog open rather than discarding in-progress input.
            return None;
        }

        let metadata = DocumentMetadata::new(trimmed_name, None, now, now)
            .expect("a non-blank trimmed name and equal created/updated times are always valid");
        let blueprint = match Blueprint::from_selection(
            layout,
            pending.selected_ids.iter().copied(),
            BlueprintId::generate(),
            metadata,
        ) {
            Ok(blueprint) => blueprint,
            Err(_) => {
                self.pending_save = None;
                return None;
            }
        };

        let result = library.save(&blueprint);
        self.pending_save = None;
        if result.is_ok() {
            self.refresh(catalog);
        }
        Some(result)
    }
}

impl Default for BlueprintLibraryView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use factory_canvas::domain::catalog::{
        BaseDefinition, BaseId, BuildableDefinition, BuildableId, CatalogId, CatalogMetadata,
        CategoryId, RegionDefinition, RegionId,
    };
    use factory_canvas::domain::geometry::{GridPoint, GridSize, Rotation};
    use factory_canvas::domain::layout::BlockInstance;
    use semver::Version;

    fn test_catalog() -> Catalog {
        let region_id = RegionId::new("view_test_region").unwrap();
        let base_id = BaseId::new("view_test_base").unwrap();
        Catalog::new(
            CatalogMetadata::new(
                CatalogId::new("view_test_catalog").unwrap(),
                Version::new(1, 0, 0),
                "View Test Catalog",
            ),
            base_id.clone(),
            vec![RegionDefinition::new(region_id.clone(), "Test Region")],
            vec![BaseDefinition::new(
                base_id,
                "Test Base",
                region_id,
                GridSize::new(20, 20).unwrap(),
            )],
            vec![BuildableDefinition::new(
                BuildableId::new("view_test_machine").unwrap(),
                "Test Machine",
                CategoryId::new("view_test_category").unwrap(),
                "TM",
                GridSize::new(2, 2).unwrap(),
                Vec::new(),
            )],
            Vec::new(),
        )
        .unwrap()
    }

    fn layout_with_one_instance(catalog: Catalog) -> (FactoryLayout, EntityId) {
        let base_id = catalog.default_base_id().clone();
        let mut layout = FactoryLayout::new(catalog, base_id).unwrap();
        let id = EntityId::new(1);
        layout
            .place(BlockInstance::new(
                id,
                BuildableId::new("view_test_machine").unwrap(),
                GridPoint::new(0, 0),
                Rotation::Zero,
            ))
            .unwrap();
        (layout, id)
    }

    #[test]
    fn disconnected_view_has_no_library_and_an_empty_listing() {
        let view = BlueprintLibraryView::new();
        assert!(view.library.is_none());
        assert!(!view.is_connected());
        assert!(view.listing().entries.is_empty());
        assert!(view.listing().invalid_entries.is_empty());

        let default_view = BlueprintLibraryView::default();
        assert!(default_view.library.is_none());
        assert!(default_view.listing().entries.is_empty());
    }

    #[test]
    fn with_library_starts_with_an_empty_cached_listing_until_refreshed() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (layout, id) = layout_with_one_instance(catalog.clone());
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let metadata = DocumentMetadata::new(
            "Pre-existing",
            None,
            OffsetDateTime::UNIX_EPOCH,
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();
        let blueprint =
            Blueprint::from_selection(&layout, [id], BlueprintId::generate(), metadata).unwrap();
        library.save(&blueprint).unwrap();

        let view = BlueprintLibraryView::with_library(library);

        assert!(
            view.listing().entries.is_empty(),
            "with_library must not perform I/O on construction"
        );
    }

    #[test]
    fn refresh_populates_listing_from_an_already_saved_blueprint() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (layout, id) = layout_with_one_instance(catalog.clone());
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let metadata = DocumentMetadata::new(
            "Saved Directly",
            None,
            OffsetDateTime::UNIX_EPOCH,
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();
        let blueprint =
            Blueprint::from_selection(&layout, [id], BlueprintId::generate(), metadata).unwrap();
        library.save(&blueprint).unwrap();

        let mut view = BlueprintLibraryView::with_library(library);
        view.refresh(&catalog);

        assert_eq!(view.listing().entries.len(), 1);
        assert_eq!(view.listing().entries[0].name(), "Saved Directly");
    }

    #[test]
    fn confirm_save_rejects_a_blank_trimmed_name_without_calling_library_save() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (layout, id) = layout_with_one_instance(catalog.clone());
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let mut view = BlueprintLibraryView::with_library(library);
        view.begin_save(vec![id]);
        *view.pending_save_name_mut().unwrap() = "   ".to_owned();

        let result = view.confirm_save(&layout, &catalog, OffsetDateTime::UNIX_EPOCH);

        assert_eq!(result, None);
        assert!(view.listing().entries.is_empty());
        assert_eq!(
            std::fs::read_dir(directory.path()).unwrap().count(),
            0,
            "a blank name must never reach the filesystem"
        );
        assert!(
            view.pending_save().is_some(),
            "a blank name is recoverable — the dialog must stay open so the \
             player can type a valid name"
        );
    }

    #[test]
    fn confirm_save_with_no_pending_save_is_a_no_op() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (layout, _id) = layout_with_one_instance(catalog.clone());
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let mut view = BlueprintLibraryView::with_library(library);

        let result = view.confirm_save(&layout, &catalog, OffsetDateTime::UNIX_EPOCH);

        assert_eq!(result, None);
        assert!(view.listing().entries.is_empty());
    }

    #[test]
    fn confirm_save_with_no_library_is_a_no_op() {
        let catalog = test_catalog();
        let (layout, id) = layout_with_one_instance(catalog.clone());
        let mut view = BlueprintLibraryView::new();
        view.begin_save(vec![id]);
        *view.pending_save_name_mut().unwrap() = "Valid Name".to_owned();

        let result = view.confirm_save(&layout, &catalog, OffsetDateTime::UNIX_EPOCH);

        assert_eq!(result, None);
        assert_eq!(
            view.pending_save().unwrap().name_input,
            "Valid Name",
            "a disconnected confirm attempt must be a true no-op"
        );
        assert!(view.listing().entries.is_empty());
    }

    #[test]
    fn confirm_save_captures_the_selection_and_appears_in_the_next_refresh() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (layout, id) = layout_with_one_instance(catalog.clone());
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let mut view = BlueprintLibraryView::with_library(library);
        view.begin_save(vec![id]);
        *view.pending_save_name_mut().unwrap() = "  Test Module  ".to_owned();

        let result = view.confirm_save(&layout, &catalog, OffsetDateTime::UNIX_EPOCH);

        assert_eq!(result, Some(Ok(())));
        assert!(view.pending_save().is_none());
        assert_eq!(
            view.listing().entries.len(),
            1,
            "confirm_save must refresh automatically on success"
        );
        assert_eq!(view.listing().entries[0].name(), "Test Module");
        assert_eq!(view.listing().entries[0].node_count(), 1);
    }

    #[test]
    fn cancel_save_creates_nothing_and_clears_pending_state() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (_layout, id) = layout_with_one_instance(catalog);
        let library = BlueprintLibrary::at(directory.path().to_path_buf());
        let mut view = BlueprintLibraryView::with_library(library);
        view.begin_save(vec![id]);
        *view.pending_save_name_mut().unwrap() = "Never Saved".to_owned();

        view.cancel_save();

        assert!(view.pending_save().is_none());
        assert!(view.listing().entries.is_empty());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
    }

    #[test]
    fn a_failed_save_leaves_the_cached_listing_untouched() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = test_catalog();
        let (layout, id) = layout_with_one_instance(catalog.clone());

        // Seed a real cached listing from a healthy library first.
        let healthy_library = BlueprintLibrary::at(directory.path().join("healthy"));
        let metadata = DocumentMetadata::new(
            "Already There",
            None,
            OffsetDateTime::UNIX_EPOCH,
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();
        let existing =
            Blueprint::from_selection(&layout, [id], BlueprintId::generate(), metadata).unwrap();
        healthy_library.save(&existing).unwrap();
        let listing_before = healthy_library.list(&catalog);
        assert_eq!(listing_before.entries.len(), 1);

        // Force library.save to fail deterministically on every supported OS:
        // its root is below a regular file, so create_dir_all cannot create
        // the required directory hierarchy. This exercises confirm_save's
        // actual Err branch without permissions, platform-specific ACLs, or
        // touching real user storage.
        let blocking_file = directory.path().join("not-a-directory");
        std::fs::write(&blocking_file, b"block child-directory creation").unwrap();
        let failing_library = BlueprintLibrary::at(blocking_file.join("blueprints"));
        let mut view = BlueprintLibraryView {
            library: Some(failing_library),
            listing: listing_before.clone(),
            pending_save: None,
        };
        view.begin_save(vec![id]);
        *view.pending_save_name_mut().unwrap() = "Cannot Be Saved".to_owned();

        let result = view.confirm_save(&layout, &catalog, OffsetDateTime::UNIX_EPOCH);

        assert!(matches!(
            result,
            Some(Err(BlueprintLibrarySaveError::Io { .. }))
        ));
        assert!(view.pending_save().is_none());
        assert_eq!(view.listing(), &listing_before);
    }
}
