# Implementation Plan: Custom Buildable Icons

**Branch**: `008-custom-buildable-icons` | **Date**: 2026-09-13 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-custom-buildable-icons/spec.md`

## Summary

Let a player associate an optional local PNG image with any buildable
through that buildable's existing catalog JSON object, so placed
instances, both kinds of placement preview, and the palette show that
icon instead of abbreviated text. A missing/unusable icon always falls
back to the existing text representation without breaking catalog
loading or editing. Icons live in one new, dedicated, project-versioned
`assets/icons/` directory — no separate image repository. The existing
placeholder orientation arrow is removed entirely: on rotation, the icon
itself (or the fallback text, when no icon is usable) turns smoothly in
its place, reusing the already-implemented `RotationVisuals` animation
system unchanged. The README gains a documented, runnable example
showing that both icons and catalog machine data are user-customizable.

Technical approach (fully detailed in [research.md](./research.md)):
add one optional `Option<Arc<str>>` field to the existing
`BuildableDefinition` domain struct and its DTO, decode PNG bytes with
`image` 0.25.10 (already compiled into this project's dependency tree
via `eframe` — promoted from transitive to direct, zero new transitive
dependencies), reuse `catalog_loader`'s already-shipped
canonicalize-and-check path-safety algorithm for the new icon directory,
load every icon into an `egui::TextureHandle` exactly once at app
startup (`FactoryCanvasApp::new`, the only call site with a live
`egui::Context`), and paint rotated icons via `egui::Image::rotate(...)`
/ rotated fallback text via `epaint::TextShape.angle` — both real
per-frame mesh/vector rotation, not pre-rasterization.

## Technical Context

**Language/Version**: Rust, stable toolchain (this session: rustc
1.97.1) — matches the rest of the project; no version change.

**Primary Dependencies**: `eframe`/`egui`/`epaint` 0.36.1 (existing, no
version change) for `Image::rotate`, `TextShape.angle`,
`Button::image_and_text`, `Context::load_texture`; `image` 0.25.10 with
`default-features = false, features = ["png"]` — promoted from an
already-resolved transitive dependency of `eframe` to a direct one (see
research.md Decision 4); no new crate is downloaded or compiled that
isn't already part of every existing build.

**Storage**: Local filesystem only, exactly like the existing catalog.
New `assets/icons/` directory (tracked, versioned with the project,
parallel to the existing `catalog/public/`), read at startup, never
embedded, never hot-reloaded — same operational model as the existing
`data/catalog/` private package (edit while closed, restart to apply).
No new persisted document fields; `FactoryDocument`/`BlueprintDocument`
schemas are untouched (FR-013).

**Testing**: `cargo test` — deterministic unit/integration tests for
domain field plumbing, catalog DTO decoding, icon-path safety
(mirroring the existing `tests/catalog_loading.rs` symlink/traversal
tests), and canvas-level rotation/preview logic, following this
project's established RED→GREEN discipline and
`docs/engineering-standards.md` §Testing scope (scoped `cargo test
--test <file>` per change, never a blanket full-suite run by default).
Manual verification of real visual appearance and interaction follows
`quickstart.md`'s seven scenarios, per this project's `conscious-orchestration`
policy of using deterministic tests as primary evidence and reserving
human visual judgment for genuinely subjective/perceptual acceptance —
not as a substitute for automated coverage.

**Target Platform**: Windows desktop (existing `factory-canvas` binary,
`eframe`/`glow` backend) — no platform change.

**Project Type**: Single native desktop application (existing
`lib.rs` + `egui_main.rs` binary + `main.rs` legacy binary structure) —
no new binary, no new crate, no workspace change.

**Performance Goals**: Not a performance-sensitive feature — icon
textures are loaded once at startup (not per-frame), and rotation
painting reuses egui's existing per-frame mesh/text rendering, which
already runs at interactive frame rates for every other canvas element.
No new performance goal beyond "does not visibly degrade the existing
editor's responsiveness," which SC-005 ("never freezes... or drops the
UI's responsiveness") already states qualitatively; no numeric target is
introduced.

**Constraints**: Fully offline (no network access for icon resolution,
per FR-005); no live reload (restart-to-apply, per FR-007); no rebuild
required for `data/catalog/`-associated icon changes, rebuild still
required only for `catalog/public/`-embedded catalog changes (unchanged
existing constraint, now explicitly extended to cover icons too, per
research.md Decision 1); per-icon raw-file-size cap before decoding
(proposed 4 MiB, research.md Decision 3) to bound worst-case resource
use from a malformed/oversized user-supplied file.

**Scale/Scope**: Same scale as the existing catalog — a handful to a few
dozen buildables in any realistic catalog (the public fallback ships 3;
a complete private catalog is not expected to reach a scale where
per-startup icon loading becomes a bottleneck). No pagination, lazy
loading, or streaming icon system is warranted at this scope (YAGNI, per
`docs/engineering-standards.md`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design below.*

| Principle | Check | Result |
|---|---|---|
| I. Explicit Engineering Standards | KISS (reuse existing patterns: `catalog_loader`'s path safety, `RotationVisuals`, `Button`'s existing AccessKit-preserving construction — no new abstraction layer); YAGNI (no live reload, no in-app editor, no asset marketplace, no animated-image support); minimal dependency (`image` already compiled in, zero new transitive deps); domain stays free of egui/filesystem (icon *string* in the domain, icon *loading/painting* entirely in `egui_app`/`egui_canvas`) | **PASS** |
| II. Architectural Decisions Live in ADRs | This feature does not contradict any existing Accepted ADR (`0001`-editor-UI, `0002`-product-name, `0003`-documents-and-blueprints). It extends `BuildableDefinition` (already the extension point ADR 0003 designed for constructible entities) with a purely presentational optional field. No existing ADR decision is reversed. **Does this warrant a new ADR?** Borderline — it is a small, additive schema extension to an already-ADR'd entity, not a new architectural direction (no new UI stack, no new persistence format, no new catalog/runtime-data boundary rule). Treated as *not* requiring a new ADR, consistent with how Phase 3's own per-instance `production_target` field (a structurally identical "add one optional field to an existing entity" change) did not spawn a dedicated ADR beyond the implementation note already added to ADR 0003. If review disagrees, a short ADR addendum is a low-cost follow-up, not a blocking gate. | **PASS (with above note)** |
| III. Spec-Driven From Commit 7 Onward | This plan follows `spec.md` (already written, clarified, and validated) via `/speckit-plan`, per the established SDD workflow every phase since Phase 5 has used. | **PASS** |
| IV. Gates Are Still Mandatory | No gate is skipped or relaxed by this plan; `tasks.md` (next command) will scope each task's test command per `docs/engineering-standards.md` §Testing scope, and the existing six-gate pre-commit sequence applies unchanged to every commit in this feature. | **PASS** |
| V. Privacy and Catalog Boundaries | `assets/icons/` ships only redistributable/user-supplied illustrative assets, never official game artwork or private reference data (FR-016); no game entity, dimension, or terminology is invented — icons are purely presentational metadata on already-confirmed public buildables. | **PASS** |

No violations requiring the Complexity Tracking table below.

## Project Structure

### Documentation (this feature)

```text
specs/008-custom-buildable-icons/
├── plan.md              # This file
├── research.md          # Phase 0 output — 8 decisions, all source-grounded
├── data-model.md         # Phase 1 output — field/type/validation shapes
├── quickstart.md         # Phase 1 output — 7 manual validation scenarios
├── checklists/
│   └── requirements.md   # Written by /speckit-specify, already complete
└── tasks.md              # Phase 2 output — NOT created by this command
```

No `contracts/` directory: this is a purely internal desktop application
with no external API, CLI surface, or service boundary this feature
touches — the existing catalog JSON schema (documented in
`docs/data-model.md`, extended per data-model.md above) already serves
as this feature's only external-facing contract, and it is fully
specified there rather than duplicated into a separate `contracts/`
artifact. This matches how prior phases (3 through 8, none of which
created a `contracts/` directory) have already handled this project's
"skip if purely internal" case.

### Source Code (repository root)

Existing single-project Rust structure (`lib.rs` + two binaries +
`tests/`), extended in place — no new top-level directory beyond the
one new asset directory:

```text
assets/
└── icons/                          # NEW — versioned, not embedded
    └── (user- and project-supplied PNG files)

catalog/
└── public/
    └── buildables.json             # gains optional "icon" field per entry

src/
├── domain/
│   └── catalog.rs                  # BuildableDefinition gains `icon` field + accessor
├── catalog_loader/
│   ├── dto.rs                      # BuildableDto gains `icon: Option<String>`
│   └── mod.rs                      # buildable-mapping closure passes icon through
├── egui_app/
│   ├── icons.rs                    # NEW — BuildableIcons, resolve_icon_path, IconPathError
│   ├── mod.rs                      # FactoryCanvasApp gains `icons: BuildableIcons` field
│   ├── startup.rs                  # new() calls BuildableIcons::load(...)
│   └── ui/
│       └── sidebar.rs              # block_palette_ui: Button::image_and_text when icon present
├── egui_canvas/
│   ├── geometry.rs                 # blueprint_preview_for_hover return type changes
│   ├── painting.rs                 # paint_instances: icon/text rotation replaces the arrow;
│   │                                # paint_orientation_arrow + orientation_arrow_direction deleted
│   └── mod.rs                      # Preview paint-layer arm gains icon/text painting;
│                                    # new CanvasFrameInput<'a> groups show()'s six read-only
│                                    # per-frame inputs (layout/title/selected/selected_block/
│                                    # armed_blueprint/icons) to stay under clippy's
│                                    # too_many_arguments limit — see tasks.md's "Deviations"
└── (egui_app/tests.rs, egui_canvas/tests.rs, tests/*.rs — updated
     alongside every touched production file, same commit, per this
     project's established compile-coupling rule)

README.md                           # new icon/data-customization subsection
```

**Structure Decision**: Extend the existing single-project layout in
place. No new crate, no workspace split, no new binary. This mirrors
exactly how Phase 3 (runtime catalog types) and Phase 8 (visual rotation
animation) each extended existing modules rather than introducing new
top-level structure, and how this repository's own prior large-file
split (the `catalog_loader/`, `egui_canvas/`, `egui_app/` module-tree
refactor immediately preceding this feature) already established the
`mod.rs` + focused-sibling-file pattern every new file above follows.

## Complexity Tracking

*No entries — the Constitution Check above recorded zero violations.*
