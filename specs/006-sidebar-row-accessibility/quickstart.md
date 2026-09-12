# Quickstart: Validating Sidebar Instance Row Accessibility

This is a validation guide, not an implementation reference — it proves
the feature works end-to-end once built. It intentionally does not
duplicate data-model.md's contracts or contain full test/method bodies;
those live in `tasks.md` and the implementation itself.

## Prerequisites

- Repository built with `cargo build --bins` (debug is enough for manual
  validation; use `cargo build --release --bins` for the release gate).
- Automated validation: `cargo test`, scoped to this feature's own files
  (`src/egui_app.rs`, `src/egui_app_tests.rs` — see
  `docs/engineering-standards.md` §Testing scope), runs the logical/
  deterministic AccessKit-tree assertions that cover every scenario below
  except the ones marked **(manual only)**, which depend on actually
  seeing the sidebar's rendered appearance, per this project's
  established preference for logical/deterministic validation over
  automated GUI capture.

## Scenario: A sidebar row is exposed as a selectable control (US1, AC1)

1. Run `cargo run --bin factory-canvas`.
2. Place one block on the canvas.

**Expected**: The instance's sidebar row is announced by assistive
technology as an interactive button, not plain text. Automated: an
editor-level test renders the sidebar with one placed instance, inspects
the AccessKit tree, and asserts the row's node has
`role() == accesskit::Role::Button` (data-model.md's AccessKit contract).
**(manual only** for the actual screen-reader announcement.)

## Scenario: Each row reports its own accurate selected state (US1, AC2-AC3)

1. Place two or more blocks.
2. Select one, then `Shift`-click to add a second to the selection,
   leaving at least one instance unselected.

**Expected**: Each selected row reports itself as selected independently
of the others, and the unselected row reports itself as not selected.
Automated: an editor-level test places three instances, selects two of
them via `SelectionMode::Add`, inspects the AccessKit tree, and asserts
`toggled() == Some(Toggled::True)` for both selected rows' nodes and
`Some(Toggled::False)` for the unselected row's node — independently,
not only for the most recently selected one.

## Scenario: Every row's complete information remains visible (US1, AC4; US2)

1. Place a block whose configured product yields the sidebar's longest
   realistic label (a long buildable/product display name).

**Expected**: The row displays the complete identifier, name, origin,
footprint, rotation, and product exactly as it does today — nothing
shortened or cut off. Automated: an editor-level test asserts the row's
AccessKit `label()` equals `instance_semantic_label`'s full output
string, byte-for-byte, for both a short and a long realistic label
(data-model.md).

## Scenario: Mouse selection still works exactly as before (FR-004)

1. With several instances placed, plain-click one row, then
   `Shift`-click another, then `Ctrl`-click a third.

**Expected**: Selection behaves exactly as it does today — plain click
replaces, `Shift` adds, `Ctrl` toggles. Automated: this is already
covered by this project's existing sidebar-selection tests (which
exercise `SelectionMode::Replace`/`Add`/`Toggle` via row clicks); re-run
them against the new widget with no expected changes to their
assertions or outcomes.

## Scenario: A focused row is visually distinguishable (US2, AC2)

1. Tab keyboard focus through the sidebar until it reaches an instance
   row.

**Expected**: The focused row is visually distinguishable from the
others (a focus-state frame/stroke change), the same way a focused block
palette option already is. **(manual only** — this depends on actually
observing the rendered frame; research.md Decision 2 explains why no new
paint code is needed for this to already be true once the widget swap
lands.)

## Scenario: Row appearance matches the block palette's affordance (US2, AC1)

1. Compare a sidebar instance row's rendered look to a block palette
   option's rendered look.

**Expected**: Both convey the same kind of selectable-control affordance
(matching frame/highlight style), since both now render through
`egui::Button`/`Button::selectable` (research.md Decision 1).
**(manual only** — a visual comparison.)

## Mechanical check: FR-005's fallback is not triggered

research.md Decision 1 already resolved this feature's one real
technical uncertainty via a disposable spike (deleted before this
document was written): `Button::wrap()` scales height with wrapped text
exactly like the current `Label::wrap()` does, at this project's actual
sidebar width and row-label format — no clipping occurs. No test asserts
the FR-005 fallback path (unchanged current rendering) is active, because
it correctly never activates for this feature.

## Scope check: `catalog/`, `data/`, `.hermes/`, domain, persistence, and every historical `specs/*` directory are unaffected

```bash
git status --short
git diff --stat
```

**Expected**: Only `src/egui_app.rs`, `src/egui_app_tests.rs`, and
`specs/006-sidebar-row-accessibility/` appear across this feature's
commits. Nothing under `catalog/`, `data/`, `.hermes/`,
`src/domain/`, `src/persistence/`, or any historical `specs/00N-*/`
directory is listed.

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
