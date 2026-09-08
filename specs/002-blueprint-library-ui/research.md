# Phase 0 Research: Blueprint Library UI

No item in Technical Context was marked `NEEDS CLARIFICATION` — every
unknown was resolvable directly from the existing codebase (Commits 5–7)
and this project's established UI conventions (`src/egui_app.rs`). This
document instead records the concrete technical decisions this plan makes,
each of which had more than one reasonable implementation but only one
that fits this codebase's existing patterns.

## Decision 1: New view-state module, not inline `FactoryCanvasApp` fields alone

**Decision**: Introduce `src/blueprint_library_view.rs`, a small,
non-`pub` struct (`BlueprintLibraryView`) owning the cached listing, the
`BlueprintLibrary` handle, and the pending-save-dialog state, with its own
`refresh`/`save` methods — mirroring `document_session.rs`'s existing
`DocumentSession` exactly (a small, egui-free struct that owns
persistence-triggering logic and is unit-testable without a live UI
context).

**Rationale**: `FactoryCanvasApp` (`src/egui_app.rs:601`) already
delegates document-save/open logic to `DocumentSession` rather than
inlining it as loose fields + free functions; this is the established
separation in this codebase between "UI shell state" and "persistence-
adjacent state with its own invariants." Following it keeps the diff small
and consistent instead of introducing a second style.

**Alternatives considered**: Add fields directly to `FactoryCanvasApp` and
free functions in `egui_app.rs`. Rejected: `egui_app.rs` is already 1,835
lines; the codebase's own precedent (`document_session.rs`, `selected_set.rs`)
is to extract cohesive, testable state into its own module once it has more
than one field and its own invariants — which this does (a cached listing
plus a pending-save sub-state).

## Decision 2: Listing is loaded once at startup and refreshed only after a successful save

**Decision**: `BlueprintLibraryView` computes its `listing` (a
`BlueprintLibraryListing`) once when `FactoryCanvasApp` is constructed, and
recomputes it exactly once immediately after `BlueprintLibrary::save`
returns `Ok`. It is never recomputed on every egui frame.

**Rationale**: `BlueprintLibrary::list` (Commit 7) performs synchronous
`fs::read_dir` plus one `fs::read` and one JSON decode per stored file.
Calling it on every repaint (potentially 60+ times/second) would scale
linearly with library size for zero benefit, since nothing outside this
same process can change the library's storage directory while the app is
running (single-user, single-process, offline-first — `docs/adr/0001`).
This mirrors how the active `Catalog` itself is loaded once at startup
(`StartupCatalog`, Commit 3) rather than re-read every frame. FR-008 only
requires the newly-saved blueprint to appear "without the player needing to
perform any extra manual refresh step" — a post-save refresh alone
satisfies this; no file-watcher or polling is needed.

**Alternatives considered**: Refresh on every frame (simplest to write, but
a real, unnecessary I/O cost that grows with library size — rejected as a
YAGNI/performance violation with no corresponding requirement). A
file-system watcher for external changes (rejected: no requirement calls
for detecting changes made by another process while this one is running,
and this product is explicitly single-process/offline-first).

## Decision 3: Save action is a contextual button inside the existing selection block

**Decision**: The "Save as blueprint" button is rendered inside
`editor_state_ui`'s existing `if selection_count > 0 { ... }` block
(`src/egui_app.rs:1364`), positioned after "Rotate 90° (R)" and before the
destructive "Remove block(s)" button.

**Rationale**: Spec Assumption #2 requires a contextual, selection-scoped
placement mirroring move/rotate/remove, which are already rendered in
exactly this block. Placing the new, non-destructive action immediately
before the red destructive "Remove" button keeps all non-destructive
actions visually grouped together and preserves "Remove" as the clear,
final, visually-distinct destructive action — consistent with this
sidebar's existing color/order convention (accent/neutral actions first,
red destructive action last).

**Alternatives considered**: An always-visible header button next to
New/Open/Save/Save As (`header_ui`, `src/egui_app.rs:1134`). Rejected:
those are document-scoped commands with no dependency on canvas selection;
mixing in a selection-scoped action there would violate the header's
existing "no selection state" contract and contradict spec Assumption #2.

## Decision 4: Library listing is a new sidebar section, not a separate window

**Decision**: Add a new "BLUEPRINT LIBRARY" section to `sidebar_ui`
(`src/egui_app.rs:1216`), after the existing editor-status section, using
the same section-header/separator convention as the base-picker and
block-palette sections above it.

**Rationale**: Directly implements spec Assumption #1 (already the
lowest-risk, explicitly-flagged default in spec.md). `sidebar_ui` already
composes three sections this way (`base_picker_ui`, `block_palette_ui`,
`editor_state_ui`) inside one shared `ScrollArea` (`src/egui_app.rs:1760`),
so appending a fourth section requires no new scrolling, windowing, or
layout mechanism.

**Alternatives considered**: A separate `egui::Window` or on-demand modal
view. Rejected per spec Assumption #1's own reasoning: introduces a new
interaction paradigm (window/panel management) this editor does not
otherwise use, for no benefit this MVP commit needs.

## Decision 5: Save dialog reuses the existing modal frame/button convention; blank names are prevented by disabling, not error text

**Decision**: `save_as_blueprint_modal` uses the exact `egui::Modal`
frame styling already used by `instance_removal_modal` and
`base_change_modal` (`SIDEBAR_BACKGROUND` fill, `BORDER` stroke, 10 corner
radius, 24 inner margin, heading + label + horizontal Cancel/Confirm
buttons), with one `ui.text_edit_singleline` for the name. The confirm
button is disabled (`ui.add_enabled(false, ...)`, matching `header_ui`'s
existing `commands_enabled` gating) whenever the trimmed name is blank,
rather than allowing submission and showing a validation error afterward.

**Rationale**: Reuses an already-reviewed, already-tested visual and
interaction pattern verbatim rather than inventing a second modal style.
FR-002/Acceptance Scenario 3 only requires that a blank-name confirmation
never submits — disabling the button is the simplest mechanism that
satisfies this with zero new state (no error-message field, no validation-
result enum) and is consistent with this codebase's existing preference
for preventing invalid actions over reporting them after the fact
(`commands_enabled` in `header_ui`).

**Alternatives considered**: Allow confirming with a blank name and show an
inline error. Rejected: adds a new UI state (error message) for a
preventable input, and this codebase has no existing precedent for
inline field-validation error text.

## Decision 6: Selection is frozen when the dialog opens, mirroring `pending_instance_removal`/`pending_base_change`

**Decision**: `PendingBlueprintSave` stores `selected_ids: Vec<EntityId>`
captured at the moment the dialog opens, not a live reference to
`self.selected`.

**Rationale**: Exactly mirrors `pending_instance_removal: Option<Vec<EntityId>>`
(`src/egui_app.rs:611`) and `pending_base_change: Option<BaseId>`
(`src/egui_app.rs:610`) — this codebase's established idiom for every
existing pending-confirmation state. Spec Assumption #3 already establishes
that the canvas selection cannot change while this modal is open (modals
block other canvas edits, consistent with `destructive_modal_open`), so
this is about idiom-consistency and defence-in-depth, not working around an
actual reachable race.

**Alternatives considered**: Re-read `self.selected` at confirm time
instead of freezing it. Rejected: inconsistent with every other pending-
action field in this struct, for no benefit.

## Decision 7: One-shot save outcomes reuse `EditorNotice`; persistent listing state does not

**Decision**: Two new `EditorNotice` variants, `BlueprintSaved` and
`BlueprintSaveFailed(BlueprintLibrarySaveError)`, are added
(`src/egui_app.rs:436`) for the one-shot outcome of a save action —
mirroring the existing `DocumentSaved`/`DocumentSaveFailed(FactoryDocumentError)`
pair exactly. The library's *persistent* contents (the entry list, empty-
library indication, invalid-entry warnings, compatibility-mismatch
indicators) are rendered directly from `BlueprintLibraryView`'s cached
`listing` inside the new sidebar section every frame — the same way
`catalog_warning: Option<String>` (`src/egui_app.rs:605`) is a persistent
banner rendered directly, never routed through the ephemeral `EditorNotice`
mechanism.

**Rationale**: `EditorNotice` already exists specifically to model "the
last thing that just happened" (a single event, replaced by the next one);
using it for the always-present listing would make the listing disappear
the next time any other notice fires, which is wrong for information the
player should be able to see at any time (FR-006, FR-007, FR-009, FR-010
are all about durable display, not one-shot events). Using two different,
already-established mechanisms for two genuinely different kinds of
information (ephemeral event vs. durable state) is the same distinction
this codebase already draws between `notice` and `catalog_warning`.

**Alternatives considered**: Route everything through `EditorNotice`,
including the listing's invalid-entry warnings. Rejected: would cause a
real, durable per-entry warning to vanish from view as soon as the player
does anything else — a regression relative to what a "safe warning" should
mean (spec User Story 3: the point is that the player can actually see and
act on it, not that it flashes once).

## Decision 8: Save-failure messages get the same privacy discipline as the library's own warnings, even though FR-008 does not literally name them

**Decision**: A new `fn safe_blueprint_save_error_detail(&BlueprintLibrarySaveError) -> String`
maps every `BlueprintLibrarySaveError` variant (`Io { kind }`,
`Encoding(_)`, `IdCollisionExhausted`) to a fixed, generic message, never
formatting or echoing `kind`'s `Debug` output or the wrapped
`BlueprintDocumentError`'s internals.

**Rationale**: FR-008 (no path/content/technical-identifier in a warning)
is written about the library's *listing* warnings, but this project's
Constitution Principle V establishes a project-wide privacy discipline for
user-facing diagnostics, and `safe_catalog_load_detail`
(`src/egui_app.rs:40`) already applies exactly this same discipline to a
different error type (`CatalogLoadError`) that FR-008 also does not
literally name. Extending the same discipline to `BlueprintLibrarySaveError`
is consistent application of an existing project-wide rule, not new scope.

**Alternatives considered**: Format `{error:?}` directly in the failure
notice (simplest to write). Rejected: would print `io::ErrorKind`'s Debug
representation or the encoding error's internals directly into a
user-facing banner, contradicting this project's established diagnostic
discipline.

## Decision 9: A missing `%LOCALAPPDATA%` degrades to a persistent notice, not a panic or silent no-op

**Decision**: If `BlueprintLibrary::default_for_user()` returns `None`
(only possible if the `LOCALAPPDATA` environment variable is entirely
unset), `BlueprintLibraryView` holds `library: None` and both the sidebar
section and the save button surface a fixed, safe "blueprint library is
unavailable" indication instead of attempting any further operation.

**Rationale**: `BlueprintLibrary::default_for_user()`'s own doc comment
(Commit 7) already states this "does not happen in practice on a real
Windows user session but is surfaced rather than papered over" — this
plan's UI must honor that same discipline instead of unwrapping or
silently ignoring it. Mirrors `catalog_warning: Option<String>`'s existing
pattern for a persistent, always-checked degraded-mode indicator.

**Alternatives considered**: `.expect(...)` and panic. Rejected: contradicts
the persistence layer's own explicit design intent to surface rather than
crash on this condition.

## Decision 10: No new dependency

**Decision**: `Cargo.toml` is not modified. `egui::TextEdit`/
`text_edit_singleline` and `egui::Modal` are already transitively available
through the existing `eframe` dependency and already used elsewhere in
`egui_app.rs`.

**Rationale**: Directly satisfies Constitution Principle I's minimal-
dependency expectation; nothing this feature needs is missing from the
existing dependency set.

**Alternatives considered**: None — no candidate dependency was ever in
consideration.

## Decision 11: Testing follows this project's existing dual-track policy, applied to the new module and new `egui_app.rs` logic

**Decision**: New logical tests are added to `src/egui_app_tests.rs`
(the only place binary-only app modules can be unit-tested — see
plan.md Project Structure), covering: name-blank-prevents-save,
selection-empty-hides-the-button, a successful save leaves canvas/selection/
next-entity-id unchanged, a successful save's blueprint appears in the
cached listing without a manual step, and invalid/duplicate library entries
render as safe text with no path/content/identifier substring. A manual
test script is written for Diogo per `docs/roadmap.md`'s existing
UI-change gate requirement, and does not block gates or publication.

**Rationale**: Directly follows this project's own documented and
memorized preference (logical/deterministic validation over automated GUI
capture; manual confirmation for real visual/interaction behavior) and the
existing precedent in `egui_app_tests.rs` for testing mutating methods and
pure helpers directly, without a live UI context.

**Alternatives considered**: Automated screenshot-diff or GUI-driver testing.
Rejected: contradicts this project's explicit, previously-stated preference
for deterministic validation over visual automation.
