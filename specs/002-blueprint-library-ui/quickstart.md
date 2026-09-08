# Quickstart: Validating Blueprint Library UI

This is a validation guide, not an implementation reference — it proves the
feature works end-to-end once built. It intentionally does not duplicate
`data-model.md`'s contracts or contain full test/method bodies; those live
in `tasks.md` and the implementation itself.

## Prerequisites

- Repository built with `cargo build --bins` (debug is enough for manual
  validation; use `cargo build --release --bins` for the release gate).
- No pre-existing blueprint library required — an empty library is itself
  one of the scenarios below (User Story 2 / FR-007).
- Automated validation: `cargo test` runs the logical/deterministic tests
  in `src/egui_app_tests.rs` (see research.md Decision 11) — this covers
  every scenario below except the ones marked **(manual only)**, which
  depend on actually seeing rendered UI and are outside what a logical test
  can assert (per this project's established preference for
  logical/deterministic validation over automated GUI capture).

## Scenario: Save the current selection as a named blueprint (US1, AC1)

1. Run `cargo run --bin factory-canvas`.
2. Place one or more blocks on the canvas; select them.
3. Click **Save as blueprint** (new contextual button in the sidebar).
4. Type a non-blank name (e.g. `"Test Module"`) in the dialog and confirm.

**Expected**: The dialog closes, a one-shot "Blueprint saved" notice
appears, the canvas still shows exactly the same instances (same IDs,
positions, rotations) as before step 3, and the sidebar's blueprint library
section now lists "Test Module" with the correct module count and a
last-saved time close to "now". **(manual only** for the actual visual
confirmation; the underlying canvas-unchanged and library-updated
assertions are covered by an automated test.)

## Scenario: Save action is unavailable with nothing selected (US1, AC2)

1. With the canvas empty or nothing selected, look at the sidebar.

**Expected**: No "Save as blueprint" button is visible (it only appears
inside the selection-scoped block, research.md Decision 3). **(manual
only** for the visual absence; covered logically by asserting the button's
enabling condition matches `selection_count > 0`.)

## Scenario: Blank name cannot be confirmed (US1, AC3)

1. Select an instance, open the save dialog.
2. Leave the name field empty (or type only spaces) and attempt to confirm.

**Expected**: The confirm button is disabled (research.md Decision 5); no
blueprint is created; the dialog stays open. Automated: assert
`confirm_save` returns `None` and performs no `library.save` call when
`name_input` is blank or whitespace-only, even if called directly
(defence-in-depth check, independent of the UI button).

## Scenario: Canceling the dialog discards it cleanly (US1, AC4)

1. Select an instance, open the save dialog, type a name, then click
   **Cancel**.

**Expected**: No blueprint is created; the canvas selection is unchanged;
re-opening the dialog afterward starts with an empty name field (no leaked
state from the canceled attempt).

## Scenario: Viewing an already-populated library (US2, AC1)

1. Ensure at least one blueprint was already saved (previous scenario).
2. Restart the app (`cargo run --bin factory-canvas` again) to prove this
   is real persistence, not in-memory-only state (ties back to Commit 7's
   own SC-001).

**Expected**: The sidebar's library section lists every previously saved
blueprint with name, module count, and last-saved time — matching
`specs/001-blueprint-library/quickstart.md`'s own restart scenario, now
visible on screen instead of only provable via `BlueprintLibrary::list`
directly.

## Scenario: Empty library shows an explicit indication (US2, AC2)

1. Run the app against a storage location with zero saved blueprints (a
   fresh profile, or point `BlueprintLibrary::at` at an empty temp
   directory in a test).

**Expected**: The library section shows an explicit "No blueprints saved
yet" (or equivalent) message — never a blank area indistinguishable from a
loading or broken state (FR-007/SC-006).

## Scenario: A new save appears without a manual refresh step (US2, AC3)

1. From a running session with the library section visible, save a new
   blueprint (US1 flow above).

**Expected**: The library section shows the new entry immediately after
the save dialog closes, with no separate "refresh" button or action needed
(FR-008/SC-002) — this is `BlueprintLibraryView::confirm_save`'s automatic
post-save `refresh` (research.md Decision 2).

## Scenario: An unreadable file surfaces as a safe warning, not a silent gap (US3, AC1–2)

1. Save one or two valid blueprints through the UI.
2. Close the app. Manually place one invalid file directly into the
   library's storage directory (`%LOCALAPPDATA%/Factory Canvas/blueprints/`),
   named like `blueprint_deadbeefdeadbeefdeadbeefdeadbeef.factory-blueprint.json`
   with content that is not valid blueprint document JSON — bypassing the
   UI entirely, exactly like `tests/blueprint_library.rs`'s own
   `invalid_file_is_isolated_while_valid_blueprints_remain_listed` test
   does at the persistence layer.
3. Restart the app and open the library section.

**Expected**: The valid blueprint(s) are still listed normally, and a safe,
generic notice (e.g. "1 entry could not be read") is also shown — reusing
`InvalidLibraryEntryReason`'s existing safe categories (Commit 7), with no
file path, raw content, or technical identifier anywhere in what is
rendered (FR-009, SC-005). This is the same guarantee
`specs/001-blueprint-library/quickstart.md` already proved at the
persistence layer, now also proved visible on screen.

## Scenario: A catalog-compatibility mismatch is visible but non-blocking (Edge Case)

1. Save a blueprint against one catalog, then start the app such that a
   different catalog data version is active (mirrors how
   `DocumentOpened(CatalogCompatibility::...)` is already exercised for
   factory documents).

**Expected**: The blueprint still appears in the listing (never hidden),
with a visible, non-blocking mismatch indication reusing
`BlueprintLibraryEntry::compatibility()` (Commit 7) — the same
`CatalogCompatibility` concept and safe presentation already used for
opened factory documents (FR-010).

## Automated edge case: Blueprint storage is unavailable (Decision 9)

This state only occurs when the process cannot resolve the user's local app-data
root. It is covered deterministically by
`disconnected_library_renders_unavailable_notice_and_disables_save_action`;
manual environment manipulation is not required.

**Expected**: The sidebar shows the fixed safe "Blueprint library unavailable."
notice. With a non-empty canvas selection, the save control is visibly disabled
and labeled "Blueprint library unavailable"; invoking the underlying request
method directly is also a no-op. The app neither panics nor silently presents a
save flow that cannot succeed.

## Running the automated checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

All six must pass before this feature's commit is considered done, per
Constitution Principle IV — unchanged from every prior commit in this
phase.
