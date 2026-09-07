# Phase 1 Data Model: Blueprint Library Persistence

Derived from `spec.md` §Key Entities and §Functional Requirements, using the
types already established in `src/domain/blueprint.rs`
(`Blueprint`, `BlueprintId`) and `src/persistence/blueprint_document.rs`
(`BlueprintDocumentError`, `encode_blueprint_document`,
`decode_blueprint_document`) from Commits 5-6. This phase adds one new
persistence-facing module; it adds no new domain type, per FR-012 (the
library is a persistence concern, not a new game/domain concept).

## New types (`src/persistence/blueprint_library.rs`)

### `BlueprintLibrary`

The library itself: a resolved storage root plus save/list behavior.

| Field | Type | Notes |
|---|---|---|
| `root` | `PathBuf` | Absolute directory this instance reads/writes. Injected via `BlueprintLibrary::at(root: PathBuf)` for tests; `BlueprintLibrary::default_for_user()` resolves `%LOCALAPPDATA%/Factory Canvas/blueprints` and calls `at` (research.md §2). |

**Validation rules**: none at construction time — `at()` never touches the
filesystem (KISS: no eager I/O in a constructor). The root is created lazily,
only inside `save()`, satisfying FR-002 ("without failing the save").

**Behavior** (signatures, not implementation):

- `fn save(&self, blueprint: &Blueprint) -> Result<(), BlueprintLibrarySaveError>`
  — encodes via `encode_blueprint_document`, resolves the collision-checked
  filename (data model item below), writes via
  `atomic_file::write_atomically`. Satisfies FR-001, FR-002, FR-010, FR-011.
- `fn list(&self, active_catalog: &Catalog) -> BlueprintLibraryListing`
  — never returns `Result`: a listing always succeeds as a whole (partial
  per-file failure is represented *inside* the returned value, not as a
  top-level error), matching User Story 2's requirement that one bad file
  never blocks the rest. Satisfies FR-003 through FR-009.

### `BlueprintLibraryListing`

The outcome of one `list()` call — deliberately two separate, already-sorted
collections rather than one mixed list, so a UI consumer (Commit 8) never has
to filter by variant itself.

| Field | Type | Notes |
|---|---|---|
| `entries` | `Vec<BlueprintLibraryEntry>` | Valid blueprints, sorted per FR-004 (name ascending, then ID ascending, then filename as a last, purely mechanical tiebreak for same-ID duplicates — research.md §5-6). |
| `invalid_entries` | `Vec<InvalidLibraryEntry>` | One entry per file that could not become a valid, unambiguous list entry — includes both genuinely malformed files and every *loser* of a duplicate-ID conflict (FR-009). |

**Validation rules**: `entries` never contains two items with the same
`BlueprintId` (that is precisely what routes the loser to
`invalid_entries` instead — FR-009).

### `BlueprintLibraryEntry`

The **summary** shown per blueprint (spec's "Blueprint Library Entry" key
entity) — deliberately not the full `Blueprint` value, so listing never
requires the UI layer to hold every node of every blueprint in memory.

| Field | Type | Notes |
|---|---|---|
| `id` | `BlueprintId` | Reused from `domain::blueprint`; the library's own identity for the blueprint (FR-004 tiebreak, FR-009/FR-010 collision key). |
| `name` | `String` | From `blueprint.metadata().name()`. Primary sort key (FR-004). |
| `node_count` | `usize` | `blueprint.nodes().len()`. Satisfies FR-003 ("module (node) count"). |
| `updated_at` | `OffsetDateTime` | From `blueprint.metadata().updated_at()`. Satisfies FR-003 ("when it was last updated"). |
| `compatibility` | `CatalogCompatibility` | Reused as-is from `persistence::factory_document` (already defined, already reviewed in Commit 6) — surfaced so a future UI can show a mismatch hint; this phase does not add new compatibility logic (spec Assumptions, last bullet). |

**Relationships**: derived 1:1 from a successfully decoded
`LoadedBlueprintDocument` (Commit 6); never constructed by hand from a
`Blueprint` skipping the codec, so every entry the library reports has
actually round-tripped through the same validated decode path used for
opening any blueprint document.

**State transitions**: none — a `BlueprintLibraryEntry` is a snapshot
produced fresh on every `list()` call; it is never mutated in place, matching
FR-012 (no edit capability in this phase).

### `InvalidLibraryEntry`

The spec's "Invalid Entry Warning" key entity — a safe, non-leaking notice.

| Field | Type | Notes |
|---|---|---|
| `reason` | `InvalidLibraryEntryReason` | Fixed, closed enum (below) — never a `String`, so a raw message can never leak (FR-008). |

No `path`, no raw bytes, no `BlueprintDocumentError` payload, and no
filename are stored on this type at all — not merely omitted from
`Display`, but structurally absent, so FR-008 holds by construction rather
than by discipline (mirrors the `BlueprintDocumentError`/
`FactoryDocumentError` privacy pattern: earlier phases learned that a type
which *can* carry a leaking field eventually gets one on some Display arm,
so here the type itself cannot).

### `InvalidLibraryEntryReason`

| Variant | When | FR |
|---|---|---|
| `UnreadableOrMalformed` | The file could not be decoded as a valid `BlueprintDocumentV1` at all (any `BlueprintDocumentError` case, collapsed to one safe reason — the specific decode failure is exactly the kind of technical detail FR-008 forbids surfacing). | FR-007, FR-008 |
| `DuplicateBlueprintId` | The file decoded successfully but lost the deterministic tiebreak against another file claiming the same `BlueprintId` (research.md §6). | FR-009 |

Two variants, not one, because User Story 2 and User Story 3 are
independently testable per the spec — a test must be able to assert *which*
situation produced a given warning without inspecting any leaked detail.

### `BlueprintLibrarySaveError`

| Variant | When | FR |
|---|---|---|
| `Io` | The underlying `atomic_file::write_atomically` call failed (directory not creatable/writable, disk full, etc.) — wraps only `atomic_file::AtomicWriteStage`/`io::ErrorKind` already-safe data, no path. | FR-002 (creation failure), FR-011 |
| `Encoding` | `encode_blueprint_document` failed (mirrors `BlueprintDocumentError::Serialization`, already unreachable in practice for a valid in-memory `Blueprint` but kept for exhaustiveness, matching the existing codec's own error contract). | FR-011 |
| `IdCollisionExhausted` | The bounded collision-retry loop (research.md §7) could not find a free ID. | FR-010 |

**Validation rules**: a `save()` call that returns any `Err` variant MUST NOT
have modified any pre-existing file in the target directory — this is
inherited for free from `atomic_file::write_atomically`'s existing guarantee
(same-directory temp file, atomic rename) and is exactly FR-011.

## Reused, unmodified types

- `domain::blueprint::{Blueprint, BlueprintId}` (Commit 5) — the library
  saves and lists `Blueprint` values; it does not define a competing
  identity or shape for a blueprint.
- `persistence::blueprint_document::{encode_blueprint_document,
  decode_blueprint_document, LoadedBlueprintDocument, BlueprintDocumentError}`
  (Commit 6) — the library's save/list operations are built entirely on top
  of this existing codec; no second JSON shape or parser is introduced.
- `persistence::factory_document::CatalogCompatibility` (Commit 1) — reused
  verbatim on `BlueprintLibraryEntry.compatibility`, per spec Assumptions.
- `persistence::atomic_file::write_atomically` (Commit 2) — the library's
  only write path.

## Filename convention (not a new "entity", but load-bearing for FR-004/FR-009/FR-010)

`<blueprint_id>.factory-blueprint.json`, where `<blueprint_id>` is exactly
`BlueprintId::as_str()` (already `blueprint_<32-hex>`, filesystem-safe with
no extra encoding needed). A file matches the library's discovery filter
only if a filename in this exact shape (prefix `blueprint_`, 32 lowercase
hex characters, exact suffix) can be parsed back through
`BlueprintId::parse` — this doubles as a cheap pre-filter before ever
opening the file for FR-005 ("ignore any file that is not a blueprint
file"), separate from and prior to the full JSON decode.
