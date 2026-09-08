# Implementation Plan: Blueprint Library UI

**Branch**: `002-blueprint-library-ui` (directory identifier only — see spec.md
header; no git branch, per constitution Workflow and Branching)
**Date**: 2026-09-07
**Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-blueprint-library-ui/spec.md`

## Summary

Add an editor UI, built entirely on top of already-shipped Commits 5
(`Blueprint::from_selection`) and 7 (`BlueprintLibrary::save`/`list`), that
lets a player (1) trigger saving the current canvas selection as a named
blueprint into the local library via a contextual button + confirmation
modal, and (2) see the library's contents — including safe warnings for
unreadable/duplicate entries and catalog-compatibility mismatches — as a
persistent section of the existing sidebar. No new domain type, no new
persistence mechanism, no new dependency: this commit is a consumer of two
already-complete, already-tested capabilities, wired into the existing
`FactoryCanvasApp` UI shell (`src/egui_app.rs`) via one new app-internal
view-state module that mirrors the established `document_session.rs`
pattern.

## Technical Context

**Language/Version**: Rust 1.97.1, edition 2021 (unchanged; matches Commit 7)

**Primary Dependencies**: `eframe`/`egui` 0.36.1 (already a dependency;
`egui::TextEdit`/`text_edit_singleline` and `egui::Modal` are already used
elsewhere in `egui_app.rs` — no new API surface from the dependency). No
addition to `Cargo.toml`.

**Storage**: Reuses `BlueprintLibrary::default_for_user()` from
`src/persistence/blueprint_library.rs` (Commit 7) as-is. This commit
introduces no new storage location, file format, or persistence mechanism.

**Testing**: `cargo test`, following this project's existing dual-track
policy for UI changes (`docs/roadmap.md` "Required gates" for UI changes):
(a) logical, deterministic unit tests in `src/egui_app_tests.rs` for every
new pure function and mutating method, mirroring how `confirm_instance_removal`,
`cancel_instance_removal`, and `production_target_action_for_choice` are
already tested without a live UI context; (b) a manual test script for
Diogo to confirm actual visual/interaction behavior, which does not block
gates or publication (per this project's established, memorized workflow
preference — logical/deterministic validation over automated GUI capture).

**Target Platform**: Windows desktop (unchanged; `factory-canvas` binary via
`src/egui_main.rs`)

**Project Type**: Desktop application, single Rust crate with a library
target (`src/lib.rs`) and two binary targets (`src/egui_main.rs`,
`src/main.rs` — the latter frozen per `docs/roadmap.md`). This feature adds
one new binary-only module and modifies one existing binary-only module;
nothing is added to the library target's public surface.

**Performance Goals**: No new performance requirement. The one
filesystem-I/O-sensitive design choice (when to call `BlueprintLibrary::list`,
which does synchronous directory + per-file reads) is resolved in research.md
Decision 2 to avoid any per-frame I/O.

**Constraints**: Must not modify `src/domain/**`, `src/persistence/blueprint_library.rs`,
`src/persistence/blueprint_document.rs`, or `src/main.rs` (frozen legacy
binary). Must not insert a blueprint into the canvas, or edit/delete/rename/
import/export a blueprint (FR-012 — reserved for a later phase). Must follow
this editor's existing visual and interaction conventions for buttons,
modals, and notices (see research.md Decisions 3–7) rather than introduce a
new UI paradigm.

**Scale/Scope**: One new file (~100–150 lines, app-internal view state), one
modified file (`egui_app.rs`: one new struct field, one new sidebar section,
one new contextual button, one new modal function, two new `EditorNotice`
variants, wiring in `ui_with_dialogs`), one line added to `egui_main.rs`,
new tests appended to `egui_app_tests.rs`. Comparable in size to Commits 4–7.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Explicit Engineering Standards**: PASS. No new external dependency.
  KISS/YAGNI honored: no per-frame filesystem I/O (research.md Decision 2),
  no new enum variant family invented where the existing `EditorNotice`
  already fits the shape of the need for one-shot save outcomes (Decision 7).
  The new module reuses an established in-repo pattern (`document_session.rs`)
  rather than inventing a new one.
- **II. Architectural Decisions Live in ADRs**: PASS. No new architectural
  decision is introduced. ADR 0001 (egui UI) and ADR 0003 (blueprints are
  independent copies, no live link to their source) are followed exactly:
  this commit only calls the existing `Blueprint::from_selection` (Commit 5)
  and `BlueprintLibrary` (Commit 7) contracts, unmodified.
- **III. Spec-Driven From Commit 7 Onward**: PASS. This plan follows an
  already-authored, checklist-validated `spec.md` for this feature.
- **IV. Gates Are Still Mandatory**: N/A at planning time — carried forward
  unchanged to implementation. All six gates (`cargo fmt --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
  `cargo build --release --bins`, `git diff --check`,
  `hermes verify --skip-start --json --timeout 300`) apply verbatim.
- **V. Privacy and Catalog Boundaries**: PASS. No `data/**`, `reference/**`,
  or `.hermes/**` content is touched or introduced. This plan explicitly
  extends the project's existing privacy discipline to a case the spec's
  FR-008 does not literally name — the new save-failure notice
  (research.md Decision 8) — rather than falling short of it.

No violations. Complexity Tracking is not applicable (empty).

**Post-Phase-1 re-check**: Confirmed after writing data-model.md and
quickstart.md below — no new domain type was introduced (the two new types
in data-model.md are app-internal view state, not domain or persistence
types), no new dependency was added, and no decision in Phase 1 contradicts
ADR 0001 or ADR 0003. The Constitution Check above still holds unchanged
post-design.

## Project Structure

### Documentation (this feature)

```text
specs/002-blueprint-library-ui/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no API, CLI, or external
service boundary — its only consumer is the in-crate egui UI added by this
same commit (same justification `specs/001-blueprint-library/plan.md` used
for Commit 7).

### Source Code (repository root)

This project is a single Rust crate with one library target (`src/lib.rs`,
covering `catalog_loader`, `domain`, `persistence`) and two binary targets
(`src/egui_main.rs` → `factory-canvas`; `src/main.rs` → frozen
`factory-canvas-legacy`). There is no web/mobile split; the generic
spec-kit "Option 1/2/3" structures do not apply as written and are replaced
below with this repository's real, existing layout:

```text
src/
├── domain/                        # EXISTING, unmodified — Blueprint (Commit 5),
│                                   # BlueprintId, FactoryLayout, EntityId, etc.
├── persistence/
│   ├── blueprint_document.rs      # EXISTING, unmodified (Commit 6)
│   └── blueprint_library.rs       # EXISTING, unmodified (Commit 7) —
│                                   # BlueprintLibrary::default_for_user/save/list
├── document_session.rs            # EXISTING, unmodified — the established
│                                   # pattern this feature's new module mirrors
│                                   # (small, testable, non-egui app state)
├── blueprint_library_view.rs      # NEW — BlueprintLibraryView, PendingBlueprintSave
│                                   # (app-layer view state; see data-model.md)
├── egui_app.rs                    # MODIFY — new FactoryCanvasApp field, new
│                                   # sidebar section, new contextual button, new
│                                   # modal, two new EditorNotice variants, wiring
├── egui_app_tests.rs               # MODIFY — new tests for the above
├── egui_main.rs                   # MODIFY — register `mod blueprint_library_view;`
└── selected_set.rs                # EXISTING, unmodified

tests/
└── (no new file) — this feature's code is binary-only (registered via `mod`
    in egui_main.rs, like document_session and selected_set already are),
    not part of the `factory_canvas` library target's public surface, so it
    cannot be reached from `tests/*.rs` integration tests (which import via
    `factory_canvas::...`, e.g. tests/blueprint_library.rs testing
    src/persistence/blueprint_library.rs). It is unit-tested in-binary via
    `#[cfg(test)]` inside src/egui_app_tests.rs, exactly like
    document_session.rs's own bottom-of-file `#[cfg(test)] mod tests`.
```

**Structure Decision**: Extend the existing binary-only app-module set
(`document_session.rs`, `selected_set.rs`, `egui_canvas.rs`) with one new
sibling module (`blueprint_library_view.rs`), and modify `egui_app.rs` to
consume it — no new crate, no new top-level directory, no change to the
library target's public API surface (`src/lib.rs` is untouched).
`BlueprintLibraryView` follows the same no-I/O-by-default,
explicit-connection shape `BlueprintLibrary` itself already established in
Commit 7 (`at()` for tests/explicit construction vs. `default_for_user()`
for the one real production caller) — see data-model.md's "Design
correction" note for why this matters for `from_startup_catalog`, the
single construction path shared by production and every unit test.

## Complexity Tracking

*(Not applicable — Constitution Check reported no violations.)*
