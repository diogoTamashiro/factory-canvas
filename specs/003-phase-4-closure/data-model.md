# Phase 1 Data Model: Phase 4 Closure Documentation

This feature ships no application code, so there is no Rust struct,
persistence format, or domain type to define. Per research.md Decision 4,
this document instead models the two kinds of content this closure must
get right, so `/speckit-tasks` has a concrete, checkable worklist instead of
an empty file.

## `DocumentTarget`

One entry per file this closure may modify. Exactly six exist; this
closure MUST NOT introduce a seventh (spec.md FR-008, SC-005).

```text
DocumentTarget
  path: repository-relative file path
  stale_passage: the specific outdated location(s) in the file today
  target_state: what that location must say once this closure is done
```

| `path` | `stale_passage` | `target_state` |
|---|---|---|
| `docs/roadmap.md` | §"Next active slice and remaining MVP phases" lists Phase 4 as upcoming; no integrated Phase 4 section exists | New "## Phase 4 — JSON documents and blueprint library — integrated" section (same format/depth as the existing integrated phase sections above it, e.g. "## Phase 3 — data package and per-entity product — integrated"); "Next active slice" reduced to Phase 5 only |
| `README.md` | Status paragraph (line 5): "...The next MVP work is versioned factory documents and local blueprints..." | States Phase 4 is integrated (save-as-blueprint + local library browsing available); next MVP work is Phase 5 |
| `CONTEXT.md` | §"Roadmap and next implementation": "Phase 4 is next: versioned FactoryDocument and BlueprintDocument persistence with atomic local saves." | States Phase 4 is integrated; Phase 5 is next |
| `docs/architecture.md` | §"CAD, documents, and runtime data" intro: "Factory and blueprint documents remain the next layers"; §"Canvas" → "Next increments": "Phase 4 adds versioned factory documents, blueprint documents, migrations, and atomic local saves" | Both passages reconciled: documents, blueprint capture, and persistence are implemented (Commits 1–8); "Next increments" instead names Phase 5 (blueprint insertion, physical ports) |
| `docs/data-model.md` | Status banner (line 3) and §"Planned factory document" / §"Planned blueprint document" / §"Planned persistence" | Banner and section headings/bodies reflect `FactoryDocument`, `BlueprintDocument`, and their persistence as implemented; physical ports and blueprint insertion remain correctly described as still planned |
| `docs/adr/0003-cad-documents-and-blueprints.md` | Has a "Phase 3 implementation note" (lines 28–32) but no Phase 4 equivalent | New dated "## Phase 4 implementation note — 2026-09-07" appended after the Phase 3 note, before "## Consequences"; original Status/Decision text unchanged (spec.md Edge Cases) |

### Validation rules

- Every `DocumentTarget.path` MUST already exist in the repository before
  this closure starts (all six were read and confirmed present during
  specification — see plan.md Technical Context).
- A `DocumentTarget`'s edit MUST NOT alter any confirmed game-data fact
  (base names, dimensions, footprints) already recorded in that file
  (spec.md FR-007).
- `docs/adr/0003-cad-documents-and-blueprints.md`'s edit is additive only
  (a new dated section); its existing "Status," "Context," and "Decision"
  headings and their current text are read-only for this closure (FR-006).

## `PhaseStatusClaim`

A single testable assertion this closure introduces or corrects: "does
document X, at location Y, correctly state that Phase 4 is integrated (or
that Phase 5 is next)?"

```text
PhaseStatusClaim
  document: DocumentTarget.path
  location: section/line identifying where the claim lives
  asserts: "Phase 4 integrated" | "Phase 5 next" | "capability C implemented"
```

Six `PhaseStatusClaim`s map 1:1 to the six `DocumentTarget` rows above (one
governing claim per file, though `docs/data-model.md` and
`docs/architecture.md` each carry two locations). Collectively they are the
literal content SC-002 and SC-004 check: after this closure, every
`PhaseStatusClaim.asserts` must be true and none may contradict another
across the six files (e.g., no file may still claim "Phase 4 is next" once
`docs/roadmap.md` says integrated).

## `CrossReference`

Any relative path or document link this closure adds or touches while
editing a `DocumentTarget` (e.g., a new roadmap-to-architecture pointer, or
preserving `README.md`'s existing links to `docs/data-model.md`).

```text
CrossReference
  source_document: DocumentTarget.path
  target_path: the path or anchor it points to
```

### Validation rule

- Every `CrossReference.target_path` introduced or left in place by this
  closure's edits MUST resolve to a file that actually exists in the
  repository (spec.md SC-003). This closure does not need to add new
  cross-references to satisfy its user stories, but must not break any it
  incidentally touches while editing surrounding text.

## Reused, unmodified existing content (for reference — not redefined here)

Confirmed already correct and left untouched by this closure:

- `docs/roadmap.md`'s "Product goal," "Non-negotiable principles,"
  "Tracked public compatibility fallback," and every already-integrated
  phase section through Phase 3.
- `docs/data-model.md`'s "Layer separation," "Identifiers," "Modular data
  package," "Constructible entity," and "Implemented positioned entity"
  sections — these already correctly describe shipped Phase 3 capability
  and are not touched.
- `docs/adr/0003-cad-documents-and-blueprints.md`'s "Status," "Context,"
  "Decision," "Phase 3 implementation note," "Alternatives considered," and
  "Review" sections.
- `docs/engineering-standards.md` and `docs/product-scope.md` — confirmed
  during research (Decision 2) to contain no Phase-4-specific stale
  language; out of this closure's six-file scope entirely.
