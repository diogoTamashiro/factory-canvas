# Implementation Plan: Visual Rotation Animation

**Branch**: `007-visual-rotation-animation` | **Date**: 2026-09-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-visual-rotation-animation/spec.md`

## Summary

Deliver Phase 8 of the roadmap: give every placed instance a small
orientation indicator (a temporary arrow, since real per-block icons do
not exist yet) and animate it smoothly whenever the domain accepts a
rotation — both a single instance turning in place and a multi-instance
orbital rotation sliding and turning together around their shared pivot.
Implemented entirely inside `egui_canvas.rs` (the module
`docs/architecture.md` already reserves for "fit, viewport, marquee,
focus, and painter") using `egui::Context`'s own built-in
`animate_value_with_time` — no new crate, no custom timing/easing code,
no `Instant`/`SystemTime` bookkeeping. The domain, catalog, and
persistence layers are entirely untouched: this is a canvas-presentation
feature that starts only after the domain has already accepted a
rotation, per spec FR-007/FR-010.

## Technical Context

**Language/Version**: Rust, stable toolchain, edition 2021 (unchanged
from every prior phase).

**Primary Dependencies**: None new. `egui::Context::animate_value_with_time`
(confirmed present and suitable via a disposable spike during this
planning phase — see research.md Decision 1) is already part of the
`egui` 0.36.1 dependency this project already has.

**Storage**: N/A — the animation/orientation state is transient,
UI-local, in-memory-only presentation state (spec.md Key Entities). It
is never part of `FactoryDocument`, `BlueprintDocument`, or any saved
file, and never touches undo/redo history (`src/history.rs`,
`EditorSnapshot`) either (FR-009).

**Testing**: `cargo test`, scoped to the files this feature actually
touches per `docs/engineering-standards.md` §Testing scope. New
integration tests drive `egui::RawInput.time` explicitly (confirmed
viable by this planning phase's spike) to assert exact interpolated
values at specific points in a transition — a deterministic, logical
test of "the value at t=0.1s of a 0.2s transition is roughly halfway
between start and end," not a screenshot or visual capture, consistent
with this project's established preference for logical/deterministic
validation over automated GUI capture.

**Target Platform**: Windows desktop (unchanged); no new platform
surface.

**Project Type**: Desktop application (single Rust crate + two
binaries), unchanged.

**Performance Goals**: N/A beyond existing invariants. `animate_value_with_time`
is O(1) per call (a single hash-map lookup inside `egui`'s own
`AnimationManager`) and this project's documented scale is tens to low
hundreds of entities (spec.md does not raise this further); repeated
rapid rotation triggers must not degrade input responsiveness (SC-005) —
confirmed structurally true because each rotation is still one
synchronous domain call exactly as today, with the animation read-only
on the painting side.

**Constraints**: A rejected rotation attempt MUST NOT start or affect
any transition (FR-005). A transition already in progress when a new
rotation is accepted MUST continue from its current visual state, not
restart or jump (FR-006) — confirmed by this planning phase's spike as
`animate_value_with_time`'s native retargeting behavior, requiring no
extra code. Changing the active layout (New/Open/base change) MUST
discard any pending transition without finishing it (FR-008). Undo/redo
MUST remain exactly as instant as it is today, with no transition
animation (FR-009, Phase 6 behavior unchanged) — this means undo/redo's
shared `apply_restored_snapshot` needs the same instant-sync treatment
as New/Open/base change, even though spec.md's Edge Cases only phrase it
as "no animation," not explicitly "discard pending transitions": without
it, restoring a snapshot with a different stored rotation would look
identical to a real rotation to this feature's own change-detection and
incorrectly animate, which is exactly the FR-009 violation the spec
edge case rules out.

**Scale/Scope**: One new type (`RotationVisuals`, inside
`egui_canvas.rs`, no new file) tracking a per-instance accumulated target
angle and origin, used to detect real domain-side changes versus a
same-value no-op; a small arrow-drawing helper alongside the existing
`paint_instances`; instrumentation of the four existing layout-replacing
call sites in `src/egui_app.rs` (`new_document_at`, `open_document_from`,
`replace_base`, and undo/redo's shared `apply_restored_snapshot`) to
instant-sync the visual state per FR-008/FR-009; and their respective
tests. Comparable in size to Phase 7 (a single rendering-path change plus
new coverage), smaller than Phase 6 (no new persisted/history-affecting
state at all).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Explicit Engineering Standards**: PASS. No new dependency; reuses
  `egui::Context::animate_value_with_time`, an existing API this
  project's own `egui` 0.36.1 dependency already ships, confirmed
  suitable by a disposable spike (`docs/engineering-standards.md`
  §YAGNI: "treat spikes as disposable" — the spike file was written,
  run, and deleted before this plan was written, same as Phase 7's).
- **II. Architectural Decisions Live in ADRs**: This feature does not
  introduce a new architectural direction — it is a rendering-detail
  addition inside the module ADR 0001 already assigns to "canvas" work,
  using an API the underlying UI framework (also chosen by ADR 0001)
  already exposes for exactly this purpose (animation). It does not
  contradict ADR 0001, 0002, or 0003; no new ADR is needed.
- **III. Spec-Driven From Commit 7 Onward**: PASS. `spec.md` is written,
  self-validated against the 16-item quality checklist (16/16 pass), and
  approved before this plan, after four clarifying questions resolved
  every ambiguity the roadmap's own one-line description left open.
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
  capacity, or port is invented. This feature reads only rotation and
  origin values the domain already validated and stored; it invents no
  new game data.

No violations. Complexity Tracking is not applicable (empty).

**Post-Phase-1 re-check**: Confirmed after writing data-model.md and
quickstart.md below — Phase 1 introduces exactly one new type
(`RotationVisuals`, holding a `HashMap<EntityId, f32>` of accumulated
target angles plus a `u64` generation counter), directly required by
spec FR-001–FR-008, no speculative field or trait beyond what those FRs
need. No dependency change. No decision contradicts ADR 0001, ADR 0002,
or ADR 0003. The Constitution Check above still holds unchanged
post-design.

## Project Structure

### Documentation (this feature)

```text
specs/007-visual-rotation-animation/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created by /speckit-plan)
```

No `contracts/` directory: this feature exposes no network API, CLI, or
cross-process service boundary — the same no-`contracts/` justification
every prior phase's plan.md already used.

### Source Code (repository root)

```text
src/
├── egui_canvas.rs               # MODIFY — new `RotationVisuals` type
│                                 # (accumulated target angle per
│                                 # EntityId + generation counter), a
│                                 # small arrow-drawing helper added to
│                                 # `paint_instances`, and the angle/
│                                 # position interpolation calls reading
│                                 # from the `egui::Context` passed to
│                                 # `show()` — the exact module
│                                 # docs/architecture.md already assigns
│                                 # "painter" work to
├── egui_app.rs                  # MODIFY — `FactoryCanvasApp` gains a
│                                 # `rotation_visuals: RotationVisuals`
│                                 # field; `new_document_at`,
│                                 # `open_document_from`, and
│                                 # `replace_base` each call its reset
│                                 # method (FR-008); `rotate_selected_clockwise`
│                                 # passes the already-accepted new
│                                 # rotation/positions through unchanged
│                                 # (FR-007 — this feature never alters
│                                 # what those methods already return or
│                                 # store)
├── egui_canvas.rs (tests mod)    # MODIFY — new deterministic tests
│                                 # using explicit `RawInput.time` values
│                                 # to assert exact interpolated angle/
│                                 # position at specific points in a
│                                 # transition
├── egui_app_tests.rs             # MODIFY — new editor-level tests for
│                                 # FR-005 (rejected rotation starts no
│                                 # transition) and FR-008 (New/Open/base
│                                 # change discards a pending transition)
└── domain/                       # UNCHANGED — no new method, field, or
                                   # type; rotation/position validation,
                                   # collision, and bounds checking are
                                   # exactly as they are today (FR-010)

catalog/                        # UNCHANGED
data/                            # UNCHANGED (ignored, private)
.hermes/                         # UNCHANGED (ignored, private)
src/history.rs                  # UNCHANGED — undo/redo stays instant,
                                 # with no transition animation (FR-009)
specs/001-blueprint-library/     # UNCHANGED — frozen historical SDD trail
specs/002-blueprint-library-ui/  # UNCHANGED — frozen historical SDD trail
specs/003-phase-4-closure/       # UNCHANGED — frozen historical SDD trail
specs/004-blueprint-insertion-interfaces/  # UNCHANGED — frozen historical SDD trail
specs/005-command-undo-redo/     # UNCHANGED — frozen historical SDD trail
specs/006-sidebar-row-accessibility/  # UNCHANGED — frozen historical SDD trail
```

**Structure Decision**: Single-crate desktop application, unchanged from
every prior phase. `RotationVisuals` lives inside `egui_canvas.rs`
rather than as a new file or a new field directly on
`FactoryCanvasApp`'s existing `CanvasState`, because it is exclusively
painting-and-interpolation state consumed only by `egui_canvas::show()`
— `egui_app.rs` only needs to reset it at three specific call sites
(FR-008), never read or compute an angle/position itself. This mirrors
`CanvasState`'s own existing placement and ownership pattern in the same
file. The domain's `FactoryLayout`/`BlockInstance`/`Rotation` gain no new
method, field, or trait: every value this feature interpolates (a
`Rotation`'s degree value, an `EntityId`'s origin) is already public and
already computed by domain calls that exist today.

## Complexity Tracking

*(Not applicable — Constitution Check reported no violations.)*
