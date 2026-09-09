# Quickstart: Validating Phase 4 Closure Documentation

This is a validation guide, not an implementation reference — it proves
this documentation closure is complete and consistent once written. It
does not duplicate `data-model.md`'s table; see that file for the exact
per-file target state.

## Prerequisites

- All six files in `data-model.md`'s `DocumentTarget` table have been
  edited: `docs/roadmap.md`, `README.md`, `CONTEXT.md`,
  `docs/architecture.md`, `docs/data-model.md`,
  `docs/adr/0003-cad-documents-and-blueprints.md`.
- No other file has been modified (verify with `git status --short` /
  `git diff --stat` — see "Scope check" below).

## Scenario: `docs/roadmap.md` reads as complete on its own (US1, AC1–2)

1. Open `docs/roadmap.md` with no other context loaded.
2. Read from the top through "Next active slice and remaining MVP phases."

**Expected**: An integrated "Phase 4 — JSON documents and blueprint
library" section appears among the other integrated phase sections,
summarizing what Commits 1–8 actually shipped (versioned `FactoryDocument`
save/open, blueprint capture, and the local blueprint library's save/list
UI). "Next active slice" now names only Phase 5 (independent insertion and
exposed interfaces) — Phase 4 no longer appears there as upcoming.

## Scenario: `README.md` and `CONTEXT.md` describe current capability accurately (US2, AC1–2)

1. Read `README.md`'s "Current status" line (top of file).
2. Read `CONTEXT.md`'s "Roadmap and next implementation" section (bottom of
   file).

**Expected**: Both state that Phase 4 is integrated and that the editor
already supports saving the current selection as a named local blueprint
and browsing the local blueprint library. Both identify Phase 5 as the
next MVP work — neither describes `FactoryDocument`/`BlueprintDocument`
persistence as still upcoming.

## Scenario: Architecture and data-contract documents show no drift (US3, AC1–2)

1. Read `docs/architecture.md`'s "CAD, documents, and runtime data" section
   and its "Canvas" → "Next increments" bullet list.
2. Read `docs/data-model.md`'s status banner (top of file) and its
   "Planned factory document" / "Planned blueprint document" / "Planned
   persistence" sections.
3. Read `docs/adr/0003-cad-documents-and-blueprints.md` end to end.

**Expected**: `docs/architecture.md` and `docs/data-model.md` both describe
`FactoryDocument`, `BlueprintDocument`, and blueprint-library persistence
as implemented, while physical ports, blueprint insertion, and undo/redo
remain correctly described as still planned for Phase 5+. ADR 0003 now
contains a dated "Phase 4 implementation note" (matching the format of its
existing "Phase 3 implementation note"), placed before "## Consequences,"
recording what was actually built and explicitly confirming blueprint
insertion and physical-port interfaces remain deferred to Phase 5. The
ADR's original "Status" and "Decision" sections are unchanged.

## Mechanical check: no targeted stale phrase remains (SC-002)

Run from the repository root:

```bash
grep -niE '(factorydocument|blueprintdocument|blueprint library).{0,60}(planned|next|remain)' \
  docs/roadmap.md README.md CONTEXT.md docs/architecture.md docs/data-model.md \
  docs/adr/0003-cad-documents-and-blueprints.md
grep -niE '(planned|next|remain).{0,60}(factorydocument|blueprintdocument|blueprint library)' \
  docs/roadmap.md README.md CONTEXT.md docs/architecture.md docs/data-model.md \
  docs/adr/0003-cad-documents-and-blueprints.md
```

**Expected**: Both commands print no matches. A match indicates a stale
passage research.md Decision 2's table did not fully correct, or a
still-correct match about a genuinely deferred item (e.g., "blueprint
insertion... remains planned") that must be hand-reviewed to confirm it is
about Phase 5 scope specifically, not Phase 4's now-shipped capability.

## Mechanical check: cross-references still resolve (SC-003)

For every relative path or document link touched while editing the six
files, confirm the target exists, e.g.:

```bash
test -f docs/data-model.md && echo OK
test -f docs/architecture.md && echo OK
test -f docs/adr/0003-cad-documents-and-blueprints.md && echo OK
```

**Expected**: Every referenced path resolves. This closure is not expected
to add brand-new cross-references, only to avoid breaking existing ones
while editing surrounding text.

## Scope check: exactly six files changed (SC-005)

```bash
git status --short
git diff --stat
```

**Expected**: Only the six paths from `data-model.md`'s `DocumentTarget`
table appear. Nothing under `src/`, `tests/`, `catalog/`, `data/`,
`.hermes/`, `specs/001-blueprint-library/`, or
`specs/002-blueprint-library-ui/` is listed (spec.md FR-008).

## Running the required gates

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

All six must pass before this feature's commit is considered done, per
Constitution Principle IV and research.md Decision 1 — unchanged from
every prior commit in this phase, even though no Rust source is touched.
