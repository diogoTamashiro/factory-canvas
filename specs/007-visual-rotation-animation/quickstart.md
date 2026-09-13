# Quickstart: Validating Visual Rotation Animation

This is a validation guide, not an implementation reference — it proves
the feature works end-to-end once built. It intentionally does not
duplicate research.md's decisions or data-model.md's field-level detail;
those live in `tasks.md` and the implementation itself.

## Prerequisites

- Repository built with `cargo build --bins` (debug is enough for manual
  validation; use `cargo build --release --bins` for the release gate).
- Automated validation: `cargo test`, scoped to this feature's own files
  (`src/egui_canvas.rs`, `src/egui_app.rs`, `src/egui_app_tests.rs` — see
  `docs/engineering-standards.md` §Testing scope), runs deterministic
  tests that drive `egui::RawInput.time` explicitly and assert exact
  interpolated values at specific points in a transition, covering every
  scenario below except the ones marked **(manual only)**, which depend
  on actually watching a smooth, continuous transition render, per this
  project's established preference for logical/deterministic validation
  over automated GUI capture.

## Scenario: A rotated block visibly turns (US1, AC1-AC3)

1. Run `cargo run --bin factory-canvas`.
2. Place one block on the canvas.
3. Select it and trigger rotation (the **Rotate 90°** button or `R`).

**Expected**: An orientation-indicator arrow is visible on the block at
all times, even before the first rotation. When rotation is triggered,
the arrow visibly turns smoothly from its old direction to the new one
over a brief, perceptible transition — it does not jump instantly.
**(manual only** for actually watching the smooth motion; the underlying
interpolated values at specific simulated time points are covered by an
automated test.)

## Scenario: A rejected rotation never animates (US1, AC4; FR-005)

1. Select an instance positioned such that rotating it would be
   rejected (for example, one already at a base edge where a rotated
   footprint would go out of bounds, if the active catalog has a
   non-square buildable; otherwise use any layout-validated rejection
   path already exercised by this project's existing rotation tests).
2. Trigger rotation.

**Expected**: Nothing visibly moves or turns; the instance's orientation
indicator stays exactly where it was. Automated: an editor-level test
asserts a rejected rotation leaves `RotationVisuals`' bookkeeping (and
therefore the painted angle) completely unchanged.

## Scenario: A rotated group visibly turns and resettles together (US2, AC1-AC2)

1. Place two or more blocks.
2. Select them together and trigger rotation.

**Expected**: Every selected instance visibly slides and turns from its
old position/orientation to the new one, all resettling together at the
same moment, matching exactly the pivot the domain's own orbital
rotation already computes. **(manual only** for watching the synchronized
motion; the exact interpolated per-instance values at specific simulated
time points are covered by an automated test.)

## Scenario: A new rotation mid-transition continues smoothly (Edge Case; FR-006)

1. With a block selected, trigger rotation, then immediately trigger it
   again before the first transition visibly finishes.

**Expected**: The second transition continues from wherever the first
one currently was — no jump backward to the old resting angle, no wait
for the first transition to finish first. Automated: a test simulates
two rotations at explicit, closely-spaced `RawInput.time` values and
asserts the second transition's start value matches the first
transition's in-flight interpolated value at that instant (research.md
Decision 1's confirmed native `egui` behavior).

## Scenario: Undo/redo, New, Open, and a base change stay perfectly instant (FR-008, FR-009)

1. Rotate a block, then immediately press `Ctrl+Z` (undo) before the
   transition visibly finishes.
2. Separately: start a rotation transition, then trigger New, Open a
   different factory, or change the active base before it finishes.

**Expected**: In every case, the instance's orientation snaps instantly
to its new resting value with no visible transition at all — undo/redo
behaves exactly as it does today (Phase 6, instant snapshot restore).
Automated: an editor-level test asserts that immediately after each of
these four operations, the next frame's painted angle already equals
the final resting value, with no intermediate interpolated value
observable.

## Mechanical check: this feature touches no domain, catalog, or persistence code

```bash
git status --short
git diff --stat
```

**Expected**: Only `src/egui_canvas.rs`, `src/egui_app.rs`,
`src/egui_app_tests.rs`, and `specs/007-visual-rotation-animation/`
appear across this feature's commits. Nothing under `src/domain/`,
`catalog/`, `data/`, `.hermes/`, `src/history.rs`, or any historical
`specs/00N-*/` directory is listed.

## Running the required gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --lib --test <files this feature touches>   # see docs/engineering-standards.md §Testing scope
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

All six must pass before any commit on this feature branch is considered
done, per Constitution Principle IV — unchanged from every prior phase,
with the test gate scoped to this feature's own changed files rather
than a blanket full-suite run (Constitution v1.1.0,
`docs/engineering-standards.md` §Testing scope).
