# Phase 1 Data Model: Blueprint Library UI

Two new types, both **app-internal view state** in the new
`src/blueprint_library_view.rs` module — not domain types, not persistence
types. Nothing is added to `src/domain/` or `src/persistence/`; both existing
layers are consumed exactly as Commits 5 and 7 shipped them.

## `BlueprintLibraryView`

Owns everything this feature needs beyond what `FactoryCanvasApp` already
tracks. One instance lives as a new field on `FactoryCanvasApp`.

```text
BlueprintLibraryView
  library: Option<BlueprintLibrary>      // None only if default_for_user() failed
  listing: BlueprintLibraryListing       // cached; see Decision 2
  pending_save: Option<PendingBlueprintSave>
```

| Field | Type | Notes |
|---|---|---|
| `library` | `Option<BlueprintLibrary>` (from `crate::persistence::blueprint_library`, Commit 7, unmodified) | `None` only when `BlueprintLibrary::default_for_user()` returned `None` (research.md Decision 9). When `None`, `refresh()` and `save()` are no-ops that leave `listing` at its `Default` (empty) and report the degraded state to the caller. |
| `listing` | `BlueprintLibraryListing` (from `crate::persistence::blueprint_library`, Commit 7, unmodified) | Cached result of the last successful `library.list(catalog)` call. Starts as `BlueprintLibraryListing::default()` (empty) before the first `refresh()`. Refreshed per research.md Decision 2 (construction + after each successful save), never per-frame. |
| `pending_save` | `Option<PendingBlueprintSave>` | `Some` exactly while the save-as-blueprint modal is open; mirrors `FactoryCanvasApp::pending_base_change`/`pending_instance_removal`'s existing `Option<...>` idiom (research.md Decision 6). |

### Methods (behavioral contract, not literal signatures)

> **Design correction (caught during `/speckit-tasks`, before any code was
> written)**: an earlier draft of this contract had a single
> `new(catalog: &Catalog) -> Self` resolve `default_for_user()` internally.
> `FactoryCanvasApp::from_startup_catalog` — the *only* construction path
> for `FactoryCanvasApp`, used by both the real production entry point
> (`FactoryCanvasApp::new(creation_context)`) and **every** unit test in
> `src/egui_app_tests.rs` (`production_test_app`, `Default`, etc.) — would
> then have transitively touched the real
> `%LOCALAPPDATA%/Factory Canvas/blueprints` directory on every single test
> run. That is a real test-hygiene bug (non-hermetic tests reading/writing
> the actual developer's user profile), not a style nitpick — Commit 7's
> own tests never do this; they always inject a temp directory via
> `BlueprintLibrary::at`. The contract below is corrected to the same
> dependency-injection shape Commit 7 already established
> (`BlueprintLibrary::at` for tests/explicit construction vs.
> `BlueprintLibrary::default_for_user` for the one real production caller),
> applied one level up.

- `new() -> Self` — **no I/O**. Starts fully disconnected: `library: None`,
  `listing: BlueprintLibraryListing::default()` (empty), `pending_save: None`.
  This is what `FactoryCanvasApp::from_startup_catalog` uses, so `Default`
  and every existing/new test remain exactly as hermetic as they are today.
- `with_library(library: BlueprintLibrary) -> Self` — stores the given,
  already-constructed library with an empty cached `listing` and no
  pending save; performs no I/O itself (no `list` call). Tests use this
  with `BlueprintLibrary::at(temp_dir)` (mirroring how Commit 7's own tests
  construct `BlueprintLibrary`) when they need a real, isolated, temp-dir-backed
  library rather than the disconnected `new()` state.
- `connect_to_default_storage(&mut self, catalog: &Catalog)` — the **only**
  place `BlueprintLibrary::default_for_user()` is ever called. If it
  resolves to `Some(library)`, replaces `self` with
  `Self::with_library(library)` and performs one `refresh(catalog)`
  (Decision 2's "one construction-time refresh", now explicit rather than
  implicit). If it resolves to `None` (Decision 9), leaves `self` in its
  current disconnected/degraded state. Called exactly once, only from the
  real `FactoryCanvasApp::new(creation_context)` production entry point —
  never from `from_startup_catalog`, `Default`, or any test.
- `refresh(&mut self, catalog: &Catalog)` — no-op if `library` is `None`;
  otherwise overwrites `listing` with a fresh `library.list(catalog)` call.
- `begin_save(&mut self, selected_ids: Vec<EntityId>)` — sets `pending_save`
  to a fresh `PendingBlueprintSave` with an empty `name_input` and the given
  frozen selection (Decision 6). Does not touch `listing`.
- `cancel_save(&mut self)` — sets `pending_save` to `None`. No side effects
  on `listing` or storage (spec FR-005).
- `confirm_save(&mut self, layout: &FactoryLayout, catalog: &Catalog, now: OffsetDateTime) -> Option<Result<(), BlueprintLibrarySaveError>>` —
  returns `None` if `pending_save` is `None` or `library` is `None`
  (nothing to do / degraded mode); otherwise: trims the pending name, and if
  blank, returns `None` without submitting anything (FR-002/SC-003 — the
  UI's disabled-button gate from research.md Decision 5 is expected to make
  this branch unreachable in practice, but `confirm_save` itself must not
  trust that and must re-check, since it is the actual safety boundary, not
  the button). On a non-blank name: calls
  `Blueprint::from_selection(layout, selected_ids, BlueprintId::generate(), metadata)`
  (Commit 5, unmodified) to build the `Blueprint` value, then
  `library.save(&blueprint)` (Commit 7, unmodified). On `Ok`, clears
  `pending_save` and calls `refresh` (Decision 2); on `Err`, clears
  `pending_save` but leaves `listing` untouched (a failed save must not
  fabricate a change to what is actually stored — FR-011). Returns
  `Some(result)` either way so the caller can drive the one-shot
  `EditorNotice` (research.md Decision 7).

### Validation rules

- A blueprint is only ever captured from a **non-empty** `selected_ids`
  (`begin_save` is only reachable from the UI when `selection_count > 0`,
  research.md Decision 3 / spec FR-003) — `Blueprint::from_selection`
  already independently rejects an empty selection
  (`BlueprintCreationError::EmptySelection`, Commit 5), so this is
  defence-in-depth, not the sole enforcement point.
- A blank (post-trim) name never reaches `library.save` (FR-002) — enforced
  in `confirm_save` itself (see above), not only by the UI's disabled
  button.
- No new validation is invented for the `Blueprint` or the stored document:
  `Blueprint::from_selection`'s existing validation (Commit 5) and
  `BlueprintLibrary::save`'s existing collision/atomicity guarantees
  (Commit 7) are reused exactly as they already exist.

## `PendingBlueprintSave`

The modal's transient input state — exists only while the save dialog is
open.

```text
PendingBlueprintSave
  selected_ids: Vec<EntityId>
  name_input: String
```

| Field | Type | Notes |
|---|---|---|
| `selected_ids` | `Vec<EntityId>` | Frozen at dialog-open time (research.md Decision 6); the exact set of canvas instances that will become the blueprint's nodes via `Blueprint::from_selection` on confirm. |
| `name_input` | `String` | Bound directly to the modal's `egui::TextEdit::singleline`. Trimmed at confirm/validation time, never trimmed on every keystroke (so the player can type a leading space transiently without it being silently stripped mid-edit). |

No state transitions beyond existence: `PendingBlueprintSave` is created
whole by `begin_save` and destroyed whole by `cancel_save` or
`confirm_save` — there is no partial/multi-step wizard state.

## Reused, unmodified existing types (for reference — not redefined here)

These already exist and are consumed as-is; listed here only to make this
feature's actual data-flow boundary explicit:

- `Blueprint`, `BlueprintId`, `BlueprintCreationError` — `src/domain/blueprint.rs` (Commit 5)
- `BlueprintLibrary`, `BlueprintLibraryListing`, `BlueprintLibraryEntry`, `InvalidLibraryEntry`, `InvalidLibraryEntryReason`, `BlueprintLibrarySaveError` — `src/persistence/blueprint_library.rs` (Commit 7)
- `CatalogCompatibility` — `src/persistence/factory_document.rs` (already reused by `BlueprintLibraryEntry::compatibility()`, Commit 7)
- `EntityId`, `FactoryLayout`, `Catalog` — `src/domain/layout.rs`, `src/domain/catalog.rs`
- `EditorNotice` — `src/egui_app.rs` (Commit 3+; gains two new variants, see plan.md)

## New `EditorNotice` variants (in `src/egui_app.rs`, not a new type/module)

```text
EditorNotice::BlueprintSaved
EditorNotice::BlueprintSaveFailed(BlueprintLibrarySaveError)
```

Added as siblings to the existing `DocumentSaved`/`DocumentSaveFailed(FactoryDocumentError)`
pair (research.md Decision 7) — not part of `BlueprintLibraryView` itself,
since `EditorNotice` already lives on `FactoryCanvasApp` and is the
established one-shot-event channel for the whole app, not something this
feature should duplicate.
