# Quickstart: Validating Command-Based Undo/Redo

This is a validation guide, not an implementation reference — it proves
the feature works end-to-end once built. It intentionally does not
duplicate data-model.md's contracts or contain full test/method bodies;
those live in `tasks.md` and the implementation itself.

## Prerequisites

- Repository built with `cargo build --bins` (debug is enough for manual
  validation; use `cargo build --release --bins` for the release gate).
- Automated validation: `cargo test`, scoped to this feature's own files
  (`src/history.rs`, `src/egui_app_tests.rs` — see
  `docs/engineering-standards.md` §Testing scope), runs the logical/
  deterministic editor-level integration tests that cover every scenario
  below except the ones marked **(manual only)**, which depend on
  actually seeing the header buttons/notices, per this project's
  established preference for logical/deterministic validation over
  automated GUI capture.

## Scenario: Undo a placement (US1, AC1)

1. Run `cargo run --bin factory-canvas`.
2. Place one block on the canvas.
3. Trigger **Undo** (header button or `Ctrl+Z`).

**Expected**: The placed block disappears; every other entity, the
active base, and the identifier allocator are exactly as they were
before the placement. Automated: an editor-level test places one
instance, snapshots the layout, undoes, and asserts the layout equals
the pre-placement snapshot (`assert_eq!`). **(manual only** for the
actual visual confirmation and notice wording.)

## Scenario: Undo each of the other five commands (US1, AC2-AC5)

1. For each of removal, move, rotation, base change, and blueprint
   insertion: perform the command, then trigger **Undo**.

**Expected**: Each command's effect is fully reversed — removed
instances reappear with their original identifiers and configuration; a
moved or rotated instance (or group) returns to its exact prior origin/
orientation; a changed base reverts along with every instance that
existed on the previous base; every entity created by a blueprint
insertion disappears as one unit. Automated: one editor-level test per
command, each performing the command, snapshotting, undoing, and
asserting layout equality with the pre-command snapshot — mirrors the
placement test's shape exactly.

## Scenario: Redo reapplies an undone command (US2, AC1)

1. Perform any one of the six commands.
2. Trigger **Undo**, then trigger **Redo**.

**Expected**: The layout returns to exactly the state it was in
immediately after the command originally executed. Automated: an
editor-level test performs a command, snapshots the post-command state,
undoes, redoes, and asserts the layout equals the post-command snapshot.

## Scenario: Redo does nothing with an empty redo history (US2, AC2)

1. Open a fresh factory (no commands performed yet, or no undo since the
   last new edit).
2. Trigger **Redo**.

**Expected**: Nothing happens — no layout change, no notice implying an
action was taken. Automated: assert the layout and allocator are
unchanged, and (if the notice's presence itself is checked) the notice
remains whatever it already was, not `EditorNotice::Redone`.

## Scenario: A new command after Undo discards redo history (US2, AC3)

1. Perform command A, then command B.
2. Trigger **Undo** (reverses B).
3. Perform a different command C.
4. Trigger **Redo**.

**Expected**: Redo does nothing — command B is no longer available to
redo, since C replaced it in the timeline. Automated: assert redo
returns `None`/leaves the layout exactly as it was after C.

## Scenario: Undo/redo through several consecutive steps (US3, AC1-AC3)

1. Perform three or more different commands in sequence, taking a
   snapshot after each.
2. Trigger **Undo** three times in a row.
3. Trigger **Redo** twice in a row.

**Expected**: Each Undo reverses exactly one more command in reverse
order, stopping cleanly with no further effect once history is
exhausted; each Redo reapplies exactly one more command in original
order. After three undos then two redos, the layout matches the snapshot
taken after the second of the three original commands. Automated: one
editor-level test performs N commands with a snapshot taken after each,
undoes/redoes a mixed sequence, and asserts the layout matches the
expected snapshot at each step.

## Scenario: Undo/redo are blocked during a destructive confirmation

1. Select one or more instances and trigger removal (opens the
   confirmation modal) without confirming or cancelling it.
2. Attempt **Undo** (button and `Ctrl+Z`).

**Expected**: Nothing happens while the modal is open — same as every
other blocked editing shortcut during a pending destructive
confirmation. Automated: assert `destructive_modal_open()` is true and
that calling `undo()`/`redo()` in that state leaves the layout and both
history stacks completely unchanged.

## Scenario: History is cleared on New/Open

1. Perform at least one command.
2. Start a new factory (or open a different factory document).
3. Trigger **Undo**.

**Expected**: Nothing happens — the new/opened factory has no undo
history from the previous session. Automated: assert `can_undo()` is
`false` immediately after `new_document_at`/`open_document_from`.

## Mechanical check: a multi-entity command is one history step, not one per entity

Automated only: perform a multi-instance move or rotation (or a
multi-node blueprint insertion), trigger **Undo** exactly once, and
assert every affected entity reverts in that single Undo — not just one
of them, and not requiring multiple Undo triggers.

## Mechanical check: the identifier allocator never moves backward through undo/redo

Automated only: place an instance (consuming one allocator value), undo
it, place a *different* instance, and assert the second placement's
identifier is not the identifier freed by the undo — the allocator kept
moving forward across the undo, per FR-009.

## Scope check: `catalog/`, `data/`, `.hermes/`, and every historical `specs/*` directory are unaffected

```bash
git status --short
git diff --stat
```

**Expected**: Only files under `src/` and
`specs/005-command-undo-redo/` appear across this feature's commits.
Nothing under `catalog/`, `data/`, `.hermes/`, or any historical
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
except that the test gate is scoped to this feature's own changed files
rather than a blanket full-suite run (Constitution v1.1.0,
`docs/engineering-standards.md` §Testing scope).
