# Quickstart: Blueprint Library Persistence

Validation guide for Commit 7 (`src/persistence/blueprint_library.rs`). See
`data-model.md` for field-level detail and `spec.md` for the full acceptance
scenarios this guide exercises. No implementation code here — this is a
run/validate guide only; the task breakdown belongs to `tasks.md`.

## Prerequisites

- Repository checked out at `master`, working tree clean.
- Rust toolchain already installed (`cargo --version` succeeds).
- No new external dependency is required (research.md §1-8) — `tempfile`,
  `serde_json`, `time`, and `uuid` are already in `Cargo.toml` from prior
  commits.

## Setup

No setup beyond a normal checkout. This module introduces no new crate
dependency, no new binary, and no new runtime configuration.

## Automated validation (what `/speckit-tasks` + `/speckit-implement` must
make pass)

Run the focused test file first, then the full suite, then the six
mandatory gates (`docs/engineering-standards.md` §Pre-commit review /
constitution Principle IV):

```bash
cargo test --test blueprint_library
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --bins
git diff --check
hermes verify --skip-start --json --timeout 300
```

Expected outcome: all commands exit 0; `cargo test` shows only new passing
tests added (no existing test starts failing); `hermes verify` reports
`"ok": true`.

## Scenario walkthroughs (map 1:1 to spec.md Acceptance Scenarios)

Each walkthrough below is phrased as what an integration test in
`tests/blueprint_library.rs` demonstrates end-to-end using a real, temporary
filesystem directory (`tempfile::tempdir()`) as the injected root — never
the real user profile.

### US1 / AC1 — Blueprint survives a restart

1. Construct a `BlueprintLibrary` at a temp root.
2. Build one valid, catalog-compatible `Blueprint` and call `save()`.
3. Drop the `BlueprintLibrary` value (simulating process exit).
4. Construct a **new** `BlueprintLibrary` at the **same** temp root
   (simulating process restart).
5. Call `list()`.
6. **Expected**: `entries` contains exactly one `BlueprintLibraryEntry` whose
   `name`, `node_count`, and `updated_at` match the saved blueprint;
   `invalid_entries` is empty.

### US1 / AC2 — First-run auto-creation

1. Construct a `BlueprintLibrary` at a temp path that does **not** yet
   exist on disk (e.g. `tempdir().path().join("blueprints")`, never
   created).
2. Call `save()` with one valid blueprint.
3. **Expected**: `save()` returns `Ok(())`; the directory now exists; a
   subsequent `list()` finds the saved blueprint.

### US2 / AC1-AC2 — One broken file never hides the rest

1. Save N valid blueprints (`N >= 2`) to a temp root.
2. Write one additional file directly (bypassing the library API) at
   `<root>/blueprint_<valid-looking-id>.factory-blueprint.json` containing
   bytes that fail `decode_blueprint_document` (e.g. truncated JSON).
3. Call `list()`.
4. **Expected**: `entries.len() == N`, all N originally-saved blueprints
   present; `invalid_entries.len() == 1` with
   `reason == InvalidLibraryEntryReason::UnreadableOrMalformed`; asserting
   on the `Debug`/`Display` output of every item in `invalid_entries`
   confirms it contains no substring of the temp directory's absolute path
   and no substring of the malformed file's raw bytes.

### US3 / AC1 — Unrelated file ignored

1. Save one valid blueprint to a temp root.
2. Write an unrelated file directly into the same root (e.g.
   `notes.txt`, or a file whose name does not match the
   `blueprint_<32-hex>.factory-blueprint.json` filter).
3. Call `list()`.
4. **Expected**: `entries.len() == 1` (the valid blueprint only);
   `invalid_entries` is empty — the unrelated file produces neither an
   entry nor a warning.

### US3 / AC2 — Symlink ignored

1. Save one valid blueprint to a temp root at path `A`.
2. Create a second temp directory containing a second valid, differently-
   named blueprint at path `B`.
3. Inside the library's root, create a symbolic link (`#[cfg(windows)]`,
   `std::os::windows::fs::symlink_file`) pointing at the blueprint file in
   `B`.
4. Call `list()`.
5. **Expected**: `entries.len() == 1` (only the blueprint actually saved
   directly in the root, path `A`); the linked-to blueprint in `B` does
   **not** appear, proving the link was skipped rather than followed.

### US3 / AC3 — Duplicate identity resolved safely

1. Save one valid blueprint (`id = X`) to a temp root.
2. Copy that same file to a second filename in the same root that still
   matches the discovery filter (same `blueprint_<id>` content, i.e. same
   `BlueprintId` inside the decoded document) — simulating a manual file
   copy.
3. Call `list()`.
4. **Expected**: `entries` contains **exactly one** entry with `id == X`
   (never two); `invalid_entries` contains exactly one entry with
   `reason == InvalidLibraryEntryReason::DuplicateBlueprintId`.

### US3 / AC4 — Save-time ID collision avoided

1. Save one valid blueprint whose `BlueprintId` is known in advance (a
   blueprint constructed with an explicitly parsed, fixed
   `blueprint_<...>` ID rather than `generate()`) to a temp root.
2. Attempt to save a second, different blueprint whose `Blueprint::id()` is
   forced (via the same fixed-ID construction path used in Commit 5/6's own
   tests) to collide with the first.
3. **Expected**: the second `save()` still returns `Ok(())`; a subsequent
   `list()` shows **two** distinct entries with two distinct IDs — the
   second call detected the collision and used a different ID rather than
   overwriting the first file. (See `research.md` §7 and Complexity
   Tracking in `plan.md` for why this scenario tests the retry path via a
   forced/fixed ID rather than mocking `uuid::Uuid::new_v4()`.)

### Edge case — Save failure/interruption preserves prior state

1. Save one valid blueprint to a temp root; capture its bytes.
2. Make the root directory read-only (Windows: clear the write ACL bit via
   `std::fs::Permissions`/`set_readonly(true)`, or target a file already
   locked open, depending on what proves reliable in CI — exact mechanism
   is an implementation task, not fixed here).
3. Attempt to save a second blueprint; expect `Err(BlueprintLibrarySaveError::Io(..))`.
4. Restore write permissions (test cleanup).
5. Re-read the first blueprint's file directly and re-run `list()`.
6. **Expected**: the first blueprint's bytes are byte-identical to what was
   captured in step 1; `list()` still shows exactly the first blueprint;
   nothing from the failed second save is visible anywhere.

## Non-goals reminder (do not add tasks for these — FR-012, spec Assumptions)

- No canvas insertion.
- No edit, delete, rename, import, or export of a stored blueprint.
- No UI (button, modal, listing panel) — that is Commit 8.
- No new catalog-compatibility logic — `CatalogCompatibility` is reused
  as-is from Commit 6.
