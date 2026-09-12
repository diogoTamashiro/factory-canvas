# Implementation Plan: Sidebar Instance Row Accessibility

**Branch**: `006-sidebar-row-accessibility` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-sidebar-row-accessibility/spec.md`

## Summary

Deliver Phase 7 of the roadmap: replace the "INSTANCES ON CANVAS" sidebar
list's plain `egui::Label` rows — sensed for clicks but exposed to
AccessKit as inert static text — with `egui::Button::selectable`, the
exact same selectable-control primitive the sidebar's own block palette
already uses one section above. A disposable spike (deleted before this
plan was written, per this project's "treat spikes as disposable" rule)
confirmed empirically that `Button::wrap()` scales its rendered height
with wrapped text exactly like the current `Label::wrap()` does — the
real technical uncertainty spec.md's FR-005 named ("if no available
control can satisfy [selectable semantics] without... clipping") is
resolved: no clipping occurs, so FR-005's fallback path is not taken.

## Technical Context

**Language/Version**: Rust, stable toolchain, edition 2021 (unchanged
from every prior phase).

**Primary Dependencies**: None new. `egui::Button` is already imported
and used elsewhere in `src/egui_app.rs` (the block palette, the header,
every action button); this feature only applies it to one more call
site.

**Storage**: N/A — this is a pure rendering/accessibility change to one
existing sidebar list; no persisted document, catalog, or session state
is read or written differently.

**Testing**: `cargo test`, scoped to the files this feature actually
touches per `docs/engineering-standards.md` §Testing scope — new
assertions in `src/egui_app_tests.rs` inspect the AccessKit tree
directly (role, `toggled` state, `is_disabled`) for each sidebar
instance row, following the exact pattern already used for the block
palette's `Role::Button`/`selected` checks and for the Phase 6 Undo/Redo
button's `is_disabled()` checks.

**Target Platform**: Windows desktop (unchanged); no new platform
surface.

**Project Type**: Desktop application (single Rust crate + two
binaries), unchanged.

**Performance Goals**: N/A beyond existing invariants — this changes
which egui widget renders each row, not how many rows render or how
often; the sidebar's existing per-frame instance-list loop
(`block_palette_ui`'s sibling, the "INSTANCES ON CANVAS" loop in
`sidebar_ui`) is unchanged in shape.

**Constraints**: Every row's complete existing information (identifier,
name, origin, footprint, rotation, configured product) MUST remain fully
visible with zero truncation (spec FR-003) — verified empirically for
this project's actual row-label format and sidebar width (264px, per
`src/egui_app.rs`'s `Panel::left("base_sidebar").exact_size(264.0)`) via
the disposable spike, which measured a real three-line wrapped label
rendering at full height with the same proportional growth the current
`Label` already exhibits (not clipped, not truncated). The existing
mouse selection modifiers (plain click / `Shift` / `Ctrl`) MUST keep
working unchanged (FR-004). No file under `catalog/`, `data/`,
`.hermes/`, or any historical `specs/*` directory may be touched.

**Scale/Scope**: One existing sidebar loop in `src/egui_app.rs`
(`sidebar_ui`'s "INSTANCES ON CANVAS" section) changes its widget from
`egui::Label` to `egui::Button::selectable`; new AccessKit-inspecting
assertions added to the existing instance-list tests in
`src/egui_app_tests.rs`. No new type, module, or file. Smaller in scope
than any prior phase — a single-widget substitution plus test coverage
for its accessibility contract.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Explicit Engineering Standards**: PASS. No new dependency,
  process, or abstraction; reuses `egui::Button::selectable`, a control
  already established in this exact file for the block palette. The
  disposable spike used to validate FR-005's clipping question was
  written, run, and deleted before this plan was authored, per
  `docs/engineering-standards.md` §YAGNI ("treat spikes as disposable").
- **II. Architectural Decisions Live in ADRs**: This is a presentation-
  layer widget substitution within the already-Accepted ADR 0001 (Rust +
  egui) decision — ADR 0001 already anticipated exactly this kind of
  refinement ("Review": "Reconsider only if a measurable spike shows that
  egui cannot meet performance, DPI, keyboard, or minimum accessibility
  requirements" — this feature's spike showed the opposite, that egui
  *can* meet the accessibility requirement, so ADR 0001 stands unchanged
  and unchallenged). No Accepted ADR addresses sidebar-row widget choice
  specifically, so no new or superseding ADR is triggered.
- **III. Spec-Driven From Commit 7 Onward**: PASS. `spec.md` is written,
  self-validated against the 16-item quality checklist (16/16 pass), and
  approved before this plan.
- **IV. Gates Are Still Mandatory**: PASS. All six gates (`cargo fmt
  --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  the tests covering this feature's own changed files per
  `docs/engineering-standards.md` §Testing scope, `cargo build --release
  --bins`, `git diff --check`, `hermes verify --skip-start --json
  --timeout 300`) apply to every commit on this branch, same as every
  prior phase.
- **V. Privacy and Catalog Boundaries**: PASS. No `data/**`,
  `reference/**`, or `.hermes/**` content is read, touched, or
  introduced; no game entity, base, product, region, dimension,
  capacity, or port is invented or altered — this feature only changes
  how already-existing, already-validated instance data is *rendered*.

No violations. Complexity Tracking is not applicable (empty).

**Post-Phase-1 re-check**: Confirmed after writing data-model.md and
quickstart.md below — Phase 1 introduces no new type at all, only a
widget substitution inside one existing method and new assertions in
existing tests, directly required by spec FR-001–FR-007. No dependency
change. No decision contradicts ADR 0001, ADR 0002, or ADR 0003. The
Constitution Check above still holds unchanged post-design.

## Project Structure

### Documentation (this feature)

```text
specs/006-sidebar-row-accessibility/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no network API, CLI, or
cross-process service boundary — the same no-`contracts/` justification
`specs/001-blueprint-library/plan.md` through
`specs/005-command-undo-redo/plan.md` already used.

### Source Code (repository root)

```text
src/
├── egui_app.rs                 # MODIFY — sidebar_ui's "INSTANCES ON
│                                # CANVAS" loop: egui::Label -> Button::
│                                # selectable, carrying the existing
│                                # click/Shift/Ctrl dispatch unchanged
├── egui_app_tests.rs            # MODIFY — new AccessKit-inspecting
│                                # assertions on the existing instance-
│                                # row tests (role, toggled, is_disabled,
│                                # full label text preserved)
├── egui_canvas.rs               # UNCHANGED — canvas painting/hit
│                                # testing is untouched; this feature is
│                                # sidebar-only (spec FR-008)
└── domain/                      # UNCHANGED — no domain, catalog, or
                                  # persistence contract changes (spec
                                  # FR-009)

catalog/                        # UNCHANGED
data/                            # UNCHANGED (ignored, private)
.hermes/                         # UNCHANGED (ignored, private)
specs/001-blueprint-library/     # UNCHANGED — frozen historical SDD trail
specs/002-blueprint-library-ui/  # UNCHANGED — frozen historical SDD trail
specs/003-phase-4-closure/       # UNCHANGED — frozen historical SDD trail
specs/004-blueprint-insertion-interfaces/  # UNCHANGED — frozen historical SDD trail
specs/005-command-undo-redo/     # UNCHANGED — frozen historical SDD trail
```

**Structure Decision**: Single-crate desktop application, unchanged from
every prior phase. This feature touches exactly one existing rendering
method (`sidebar_ui`'s instance-list loop) and its existing test
coverage — no new module, no new domain concept, no new editor-session
state. The smallest-scope phase of this project to date.

## Complexity Tracking

*(Not applicable — Constitution Check reported no violations.)*
