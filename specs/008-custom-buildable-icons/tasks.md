---

description: "Task list template for feature implementation"
---

# Tasks: Custom Buildable Icons

**Input**: Design documents from `/specs/008-custom-buildable-icons/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md (all present; no `contracts/` — purely internal desktop app, see plan.md)

**Tests**: INCLUDED. This project's Constitution Principle IV ("Gates Are
Still Mandatory") and `docs/engineering-standards.md` §TDD require
RED→GREEN discipline for domain/persistence work and deterministic
logical tests for UI behavior in place of visual automation
(`docs/roadmap.md` §4: "cover transitions, rejections, and the semantic
representation with logical, deterministic tests; do not use visual
automation as a substitute for these contracts"). Every prior phase in
this repository (3 through 8, plus the module-split refactor) followed
this discipline; this feature does the same. Tests are ordered as
vertical tracer bullets (one RED test immediately followed by the GREEN
task that satisfies it), per this project's `test-driven-development`
skill — not a separate up-front test phase.

**Organization**: Tasks are grouped by user story (spec.md's US1-US4) to
enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an
  incomplete task)
- **[Story]**: Which user story this task belongs to (US1-US4)
- Every description includes an exact file path

## Path Conventions

Single Rust project (existing `src/`, `tests/` at repository root, plus
one new `assets/` directory) — see plan.md's Project Structure section
for the full file map.

## Deviations from plan.md / data-model.md discovered during implementation

- `BuildableDefinition::new`'s new `icon` parameter is `Option<&str>`,
  not `data-model.md`'s originally proposed `Option<impl Into<Arc<str>>>`.
  T004's implementation hit `error[E0283]: type annotations needed:
  cannot infer type of the type parameter T declared on the enum
  Option` at every call site that passes a bare `None` — a generic
  `Option<impl Into<Arc<str>>>` has no way to infer `T` from `None`
  alone. `Option<&str>` fixes this (`None::<&str>` is never needed) and
  still converts to `Arc<str>` internally via `icon.map(Into::into)`.
  `data-model.md`'s Domain layer section is stale on this one signature
  detail; every other aspect of that section (field type, accessor
  shape, validation boundary) is unchanged and accurate.
- `egui_canvas::show`'s signature gained a new `icons: &BuildableIcons`
  parameter (T035), which pushed its argument count to 8 — above
  clippy's `too_many_arguments` default limit of 7, failing the
  `-D warnings` gate. Per this project's own documented rule ("fold
  into an existing state struct, not `#[allow]`"), `layout`, `title`,
  `selected`, `selected_block`, `armed_blueprint`, and `icons` (six
  read-only per-frame inputs, none of them `CanvasState`'s own
  mutable cross-frame state) were grouped into a new
  `pub(crate) struct CanvasFrameInput<'a>` in `src/egui_canvas/mod.rs`,
  reducing `show(ui, input, state)` to 3 parameters. This is a new
  type not mentioned in `plan.md`/`data-model.md`; it is presentation
  plumbing only (no new behavior, no new state), so those documents
  are not otherwise stale.
- T043 (wiring `BuildableIcons`'s warnings into the existing
  notice/warning surface) was completed during US1's own T036 gate
  pass, ahead of its originally-planned US2 phase — clippy's
  `dead_code` lint on the `warnings` field required a real production
  call site to exist before `-D warnings` could pass, and the correct
  fix (per this project's `rust-egui-desktop-app` skill entry
  "Clippy dead-code on the production target signals unfinished UI
  wiring, not a false positive") is to finish the wiring, not silence
  the lint. `BuildableIcons` gained a `first_warning() -> Option<&str>`
  accessor (surfacing one warning at a time, alongside
  `catalog_warning`) and `src/egui_app/ui/header.rs` now renders it the
  same way `catalog_warning` already is. US2's own T043 checklist line
  is left checked with this note rather than removed, since the
  described integration now already exists.
- T038/T039, T040/T041, and T042 (US2) each found their target
  behavior ALREADY correct when the new test was written and run —
  `BuildableIcons::load`'s failure isolation, its silent-vs-warned
  distinction for blank/absent vs. unusable references, and
  `resolve_icon_path`'s traversal rejection wired end-to-end through
  `load()` were all already implemented as part of T030's original
  `load()` body (US1), written defensively per spec.md FR-004's
  contract even though US1's own tests only exercised one buildable at
  a time. Each new US2 test passed immediately on its first run — no
  RED phase was observed for these three, and no production code
  changed for T039/T041 as a result; only the new test functions
  themselves are new files/lines. Per this project's TDD discipline,
  a test that passes immediately on first run is normally suspect
  (it may test the wrong thing); here the specific assertions were
  checked by hand against `load()`'s already-read source (the
  `if icon.is_empty() { continue; }` silent-skip and the per-iteration
  `match ... Err(error) => warnings.push(...)` isolation are both
  visible in the existing code, not inferred), which is why they were
  kept as real regression coverage rather than discarded as
  tautological. T044 (a non-string `icon` JSON value) needed no new
  production code either — `deny_unknown_fields` plus serde's ordinary
  type-mismatch error already reject it as `CatalogJsonErrorKind::Schema`.
- `BuildableIcons::empty()` and `BuildableIcons::load(...)` widened from
  `pub(super)` to `pub(crate)` during US3 (T053) — `egui_canvas`'s own
  test module needed to construct a `BuildableIcons` with a real loaded
  texture to genuinely exercise `paint_instances`'s `Icon` branch,
  which `pub(super)` (visible only inside `egui_app`) did not allow.
  This does not widen production-code access at all: `BuildableIcons`
  itself was already `pub(crate)`, and its only production callers
  remain inside `egui_app` (`FactoryCanvasApp::new`); the change only
  lets a second, already-`pub(crate)`-accessible module's *tests*
  construct one directly, matching `.texture()`'s existing `pub(crate)`
  visibility.
- `data-model.md`'s breaking-change section for `blueprint_preview_for_hover`
  proposed a plain returned struct with no associated painting helper.
  Implementation (T051/T052) additionally introduced
  `paint_preview_representation` in `src/egui_canvas/painting.rs`
  (`pub(super)`, not in any spec artifact) — a small function factoring
  out the icon-or-text-at-rest-orientation-and-opacity painting shared
  by both the single-buildable and blueprint-member preview call
  sites, per research.md Decision 7's own stated intent ("extending
  call sites... KISS/DRY... argues against introducing a second,
  icon-specific painting path"). This is presentation plumbing only —
  no new spec-visible behavior — so no spec.md/data-model.md content
  is stale, only the file list in plan.md's Project Structure.

## Pre-existing scope correction (found during this task breakdown)

`research.md`/`data-model.md` referenced `BuildableDefinition::new`'s
call sites only illustratively. A full repository search this session
found **22 real call sites across 10 files** (all test-only — the sole
*production* call site is `src/catalog_loader/mod.rs`'s buildable-mapping
closure, already covered by data-model.md):

- `tests/support/mod.rs` (1 call site — a shared helper reused by
  `tests/domain_blueprint.rs`, `tests/domain_layout_editing.rs`,
  `tests/domain_placement.rs`, `tests/domain_layout_model.rs`, so
  updating it alone fixes those four files' indirect usage)
- `tests/domain_blueprint.rs` (2 direct call sites, in addition to using `support`)
- `tests/domain_runtime_catalog.rs` (7 call sites)
- `tests/domain_production_target.rs` (1 call site)
- `tests/factory_document_codec.rs` (1 call site)
- `tests/blueprint_document_codec.rs` (1 call site)
- `tests/blueprint_library.rs` (1 call site)
- `src/blueprint_library_view.rs`'s `mod tests` (1 call site)
- `src/history.rs`'s `mod tests` (1 call site)
- `src/egui_app/tests.rs` (4 call sites)
- `src/egui_canvas/tests.rs` (1 call site)

All 11 non-`support.rs` files above are listed individually as
Foundational tasks (T005-T015) so this scope is not silently dropped.

---

## Phase 1: Setup

**Purpose**: Dependency and asset-directory groundwork, no domain/behavior change yet.

- [X] T001 Add `image = { version = "0.25.10", default-features = false, features = ["png"] }` as a direct dependency in `Cargo.toml` (research.md Decision 4 — already resolved transitively via `eframe`; this only promotes it to direct so `factory-canvas`'s own code can `use image::...`). Run `cargo check` and confirm `Cargo.lock`'s resolved `image` version does not change.
- [X] T002 [P] Create the `assets/icons/` directory (research.md Decision 1) with a short `assets/icons/README.md` stub explaining its purpose (one dedicated, versioned, non-embedded image directory — full documentation lands in README.md at T056) so the directory is tracked by Git even before any real icon exists.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain field, catalog-loader wiring, and safe path resolution
that every user story depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

### Domain field

- [X] T003 [P] Write a failing test in `tests/domain_runtime_catalog.rs` asserting a `BuildableDefinition` built with `icon: None` returns `None` from a new `.icon()` accessor, and one built with `Some("x.png")` returns `Some("x.png")`.
- [X] T004 Add the `icon: Option<Arc<str>>` field, `pub fn icon(&self) -> Option<&str>` accessor, and a new trailing `icon: Option<impl Into<Arc<str>>>` parameter to `BuildableDefinition::new(...)` in `src/domain/catalog.rs` (data-model.md's Domain layer section). Makes T003 pass.

### Call-site updates (all [P] — different files, all depend on T004; see scope correction above)

- [X] T005 [P] Update the `BuildableDefinition::new` call site in `tests/support/mod.rs` to pass `None` for the new `icon` parameter.
- [X] T006 [P] Update the 7 `BuildableDefinition::new` call sites in `tests/domain_runtime_catalog.rs` to pass `None` (except where T003 above intentionally passes `Some(...)`).
- [X] T007 [P] Update the 2 direct `BuildableDefinition::new` call sites in `tests/domain_blueprint.rs` to pass `None`.
- [X] T008 [P] Update the `BuildableDefinition::new` call site in `tests/domain_production_target.rs` to pass `None`.
- [X] T009 [P] Update the `BuildableDefinition::new` call site in `tests/factory_document_codec.rs` to pass `None`.
- [X] T010 [P] Update the `BuildableDefinition::new` call site in `tests/blueprint_document_codec.rs` to pass `None`.
- [X] T011 [P] Update the `BuildableDefinition::new` call site in `tests/blueprint_library.rs` to pass `None`.
- [X] T012 [P] Update the `BuildableDefinition::new` call site in `src/blueprint_library_view.rs`'s `mod tests` to pass `None`.
- [X] T013 [P] Update the `BuildableDefinition::new` call site in `src/history.rs`'s `mod tests` to pass `None`.
- [X] T014 [P] Update the 4 `BuildableDefinition::new` call sites in `src/egui_app/tests.rs` to pass `None`.
- [X] T015 [P] Update the `BuildableDefinition::new` call site in `src/egui_canvas/tests.rs` to pass `None`.
- [X] T016 Run `cargo check --all-targets` from the repository root (no single file — whole-crate compile gate; per this project's "check compile coupling first" rule) and confirm zero compile errors before continuing.

### Catalog loader wiring

- [X] T017 [P] Write a failing test in `src/catalog_loader/tests.rs` asserting a `BuildableDto` JSON fixture with an `"icon": "x.png"` key decodes to `Some("x.png")`, one with no `icon` key decodes to `None`, and one with `"icon": null` decodes to `None`.
- [X] T018 Add `#[serde(default)] pub(super) icon: Option<String>` to `BuildableDto` in `src/catalog_loader/dto.rs` (data-model.md's Catalog loader layer section). Makes T017 pass.
- [X] T019 Pass `buildable.icon` through to `BuildableDefinition::new(...)`'s new parameter in `src/catalog_loader/mod.rs`'s buildable-mapping closure (the sole production call site).
- [X] T020 [P] Write a failing test in `src/catalog_loader/tests.rs` using a `DirectorySource` fixture with one buildable declaring `icon` and one omitting it, asserting `load_catalog_from_directory(...)` succeeds and both buildables' `.icon()` values match expectations.
- [X] T021 Run `cargo test --test catalog_loading` (covers `tests/catalog_loading.rs`) plus `cargo check --bin factory-canvas --tests` (covers `src/catalog_loader/tests.rs`), confirming T017/T020 pass and every pre-existing catalog_loader test still passes.

### Safe icon-path resolution (pure, no I/O side effects beyond `fs::canonicalize`/`fs::metadata`)

- [X] T022 [P] Write a failing test in a new `src/egui_app/icons.rs` test module asserting `resolve_icon_path` rejects an empty string, a string containing a NUL byte, a rooted path (`/x.png`), a Windows-drive-prefixed path (`C:\x.png`), and any path containing a `.` or `..` component — mirroring `src/catalog_loader/parsing.rs`'s `validate_module_path` test coverage (research.md Decision 3).
- [X] T023 [P] Write a failing test asserting `resolve_icon_path` rejects a path that canonicalizes outside the icons root, including via a symlink/junction escape — reusing `tests/catalog_loading.rs`'s existing `create_file_symlink` + `ERROR_PRIVILEGE_NOT_HELD` skip pattern, adapted to an icons-root fixture.
- [X] T024 Implement `resolve_icon_path(icons_root: &Path, relative: &str) -> Result<PathBuf, IconPathError>` and the `IconPathError` enum in new file `src/egui_app/icons.rs` (data-model.md's `IconPathError` section), reusing `catalog_loader`'s canonicalize-then-`starts_with` algorithm. Makes T022 and T023 pass.
- [X] T025 [P] Write a failing test in `src/egui_app/icons.rs` asserting a raw file whose byte length exceeds the proposed 4 MiB cap is rejected by a new size-check function before any decode is attempted (a temp file of `cap + 1` bytes is enough; content does not need to be a valid PNG).
- [X] T026 Implement the raw-file-size cap check in `src/egui_app/icons.rs`, called before decoding (research.md Decision 3's resource-cap paragraph). Makes T025 pass.
- [X] T027 Declare `mod icons;` in `src/egui_app/mod.rs`, add `pub(super) struct BuildableIcons { textures: BTreeMap<BuildableId, egui::TextureHandle>, warnings: Vec<String> }` with a `BuildableIcons::empty() -> Self` constructor (empty map, empty warnings) in `src/egui_app/icons.rs`, and add an `icons: BuildableIcons` field to `FactoryCanvasApp` in `src/egui_app/mod.rs`, set to `BuildableIcons::empty()` in `from_startup_catalog` (`src/egui_app/startup.rs`) and in `impl Default for FactoryCanvasApp` (`src/egui_app/mod.rs`).

**Checkpoint**: Foundation ready — the domain stores an icon path, the
catalog loader decodes it, `resolve_icon_path` safely validates it, and
`FactoryCanvasApp` has an (empty) icon cache. Nothing decodes a real PNG
or paints an icon yet; every existing test still passes.

---

## Phase 3: User Story 1 (P1) 🎯 MVP — Recognize a placed buildable by a custom icon

**Goal**: A player associates a PNG with an existing buildable; after
restart, every placed instance of that type shows the icon instead of
its abbreviated text.

**Independent Test**: Associate a user-supplied image with the Xiranite
Power Pole, leave another buildable without an image, restart, place
both — the first shows its icon, the second its text (quickstart.md
Scenario 1).

- [X] T028 [P] [US1] Write a failing test in `src/egui_app/icons.rs` asserting a small pure PNG-decode helper (e.g. `decode_png_rgba`) rejects non-PNG bytes (e.g. a JPEG magic-byte fixture) and, given valid PNG bytes, returns dimensions and pixel data matching a known small fixture (research.md Decision 4 — `image::load_from_memory_with_format(bytes, ImageFormat::Png)` rejects by content, not extension).
- [X] T029 [US1] Implement `decode_png_rgba` in `src/egui_app/icons.rs` using `image::load_from_memory_with_format(..., image::ImageFormat::Png)` + `.to_rgba8()`, returning an `egui::ColorImage` via `ColorImage::from_rgba_unmultiplied`. Makes T028 pass.
- [X] T030 [US1] Implement `BuildableIcons::load(ctx: &egui::Context, catalog: &Catalog, icons_root: &Path) -> Self` in `src/egui_app/icons.rs`: for each `catalog.buildables()` entry with `Some(icon)`, resolve via `resolve_icon_path` (T024), check the size cap (T026), decode via `decode_png_rgba` (T029), and on success call `ctx.load_texture(name, color_image, TextureOptions::LINEAR)`, storing the handle keyed by `BuildableId`; any failure at any step appends one sanitized warning string to `warnings` and omits that buildable from `textures` (no failure aborts the loop).
- [X] T031 [US1] Implement `BuildableIcons::texture(&self, id: &BuildableId) -> Option<&egui::TextureHandle>` in `src/egui_app/icons.rs`.
- [X] T032 [US1] Call `BuildableIcons::load(&creation_context.egui_ctx, app.layout.catalog(), Path::new("assets/icons"))` from `FactoryCanvasApp::new` in `src/egui_app/startup.rs`, storing the result in the new `icons` field (the only call site with a live `egui::Context`, per research.md Decision 5 — `from_startup_catalog`/`Default` remain untouched, still using `BuildableIcons::empty()`).
- [X] T033 [P] [US1] Write a failing test in `src/egui_canvas/tests.rs` (or a new sibling test module) for a small pure chooser function (e.g. `fn paint_choice(texture: Option<&TextureHandle>) -> PaintChoice { Icon | Text }`) asserting it returns `Icon` when a texture is `Some` and `Text` when `None`.
- [X] T034 [US1] In `src/egui_canvas/painting.rs`: delete `paint_orientation_arrow` and `orientation_arrow_direction` entirely (spec.md FR-009); add the paint-choice helper from T033; in `paint_instances`, replace the deleted arrow call with a branch that either paints the resolved icon via `egui::Image::from_texture(&texture).rotate(angle.to_radians(), egui::Vec2::splat(0.5)).paint_at(ui, screen_rect)` (research.md Decision 6) or paints the fallback label via a new `epaint::TextShape` with `angle: angle.to_radians()`, computed from `screen_rect.center() - galley.size() * 0.5` and `painter.add(egui::Shape::Text(text_shape))` (research.md Decision 6 — `painter.text(...)` cannot rotate).
- [X] T035 [US1] Thread a `&BuildableIcons` lookup into `paint_instances`'s signature and its call site inside `src/egui_canvas/mod.rs`'s `CanvasPaintLayer::Instances` arm; update `src/egui_app/ui/canvas.rs`'s call into `crate::egui_canvas::show(...)` to pass `&self.icons` through (widen `show`'s own signature the same way, per this project's documented `too_many_arguments`/existing-state-struct rule if the parameter count becomes a concern — fold into `CanvasState` if so).
- [X] T036 [US1] Run `cargo check --bin factory-canvas --tests` (no single file — whole-binary compile+test gate), then `cargo test --bin factory-canvas` and the targeted `egui_app`/`egui_canvas`/`catalog_loader` test commands from T016/T021, confirming every new test from this phase passes and zero existing test regressed.
- [ ] T037 [US1] Manual validation: Diogo runs quickstart.md Scenario 1 end-to-end and confirms the icon actually renders correctly on screen (visual acceptance is non-blocking per this project's `conscious-orchestration` policy — report the result, do not block the gate on it).

**Checkpoint**: User Story 1 is fully functional — an icon-bearing
buildable shows its icon on placed canvas instances. This is the
suggested MVP stopping point.

---

## Phase 4: User Story 2 (P1) — Keep editing when an icon is absent or unusable

**Goal**: Text-only and partially-illustrated catalogs keep working;
missing/broken/unsafe icon references never break editing.

**Independent Test**: A catalog with one working icon and several
absent/unusable icon references — only the affected representations
fall back, every buildable stays usable (quickstart.md Scenarios 2-3).

- [X] T038 [P] [US2] Write a failing test in `src/egui_app/icons.rs` asserting `BuildableIcons::load` with one buildable pointing at a missing file and another pointing at a valid PNG produces exactly one warning and leaves the valid buildable's texture loadable (confirms per-icon failure isolation, already implemented by T030 — this test may already pass; if so, per TDD's own rule, treat a same-session pass as evidence T030 already covers it rather than writing throwaway code to force a fail).
- [X] T039 [US2] If T038 fails, fix `BuildableIcons::load`'s failure-isolation logic in `src/egui_app/icons.rs` until it passes; if T038 already passes, skip this task and note so in the commit message.
- [X] T040 [P] [US2] Write a failing test in `src/egui_app/icons.rs` asserting an omitted `icon` field, an explicit `icon: null`, and an explicit `icon: ""` all produce **zero** warnings, while a non-empty-but-unusable reference (e.g. a missing filename) produces exactly one warning — the silent-vs-warned distinction spec.md's clarified FR-004/FR-012 requires.
- [X] T041 [US2] Adjust `BuildableIcons::load`'s warning logic in `src/egui_app/icons.rs` so only a non-empty, explicit-but-unusable `icon` value warns (an empty string is treated identically to `None` before any path resolution is attempted). Makes T040 pass.
- [X] T042 [P] [US2] Write a failing test in `src/egui_app/icons.rs` asserting `BuildableIcons::load`, given a buildable whose `icon` is a parent-directory-escaping or absolute-path string pointing at a real file outside `assets/icons/`, leaves that buildable with no texture and exactly one sanitized warning — an end-to-end confirmation that `resolve_icon_path`'s rejection (T023) is actually wired into the load path, not merely unit-tested in isolation.
- [X] T043 [US2] Wire `BuildableIcons`'s collected `warnings` into the existing warning/notice surface: extend `src/egui_app/notices.rs` (or `src/egui_app/mod.rs`, matching how `catalog_warning: Option<String>` is already surfaced) so icon warnings are visible through the same mechanism, joined or listed alongside any existing catalog warning without replacing it.
- [X] T044 [P] [US2] Write a failing test in `src/catalog_loader/tests.rs` asserting a `buildables.json` fixture with a non-string `icon` value (e.g. a JSON number) still produces the existing `CatalogLoadError::InvalidJson { kind: CatalogJsonErrorKind::Schema, .. }` behavior, unchanged by this feature.
- [X] T045 [US2] Run the full scoped test set from T036 (no single file — whole-binary test gate) plus T038-T044's new tests, confirming everything passes together.
- [ ] T046 [US2] Manual validation: Diogo runs quickstart.md Scenarios 2 and 3 end-to-end.

**Checkpoint**: User Stories 1 AND 2 both work — icons show when usable,
every failure mode safely falls back to text with correctly
silent-vs-warned behavior, and nothing can read outside `assets/icons/`.

---

## Phase 5: User Story 3 (P2) — Keep recognition and orientation consistent while editing

**Goal**: Icons appear consistently across the palette and both preview
kinds; rotation turns the icon or fallback text smoothly with no arrow
anywhere; rejected/plain-move/whole-layout-restore behavior is
unaffected.

**Independent Test**: A mixed icon/text layout and a blueprint
containing both kinds — palette, both preview kinds, accepted/rejected
single and group rotation, rapid turns, and layout restoration all
behave per spec (quickstart.md Scenarios 5-6).

- [X] T047 [P] [US3] Write a failing test in `src/egui_app/tests.rs` asserting the block palette exposes an icon-bearing buildable's row with an accessible name/role matching today's text-only row contract (reusing this project's established AccessKit node-lookup pattern for sidebar/palette rows), and that a buildable without an icon still renders as a plain text-only control.
- [X] T048 [US3] Update `block_palette_ui` in `src/egui_app/ui/sidebar.rs`: when `self.icons.texture(&buildable_id)` is `Some`, build the row via `Button::image_and_text(egui::Image::from_texture(texture), label).selected(selected)`; otherwise keep the existing `Button::new(label).selected(selected)` unchanged. Makes T047 pass.
- [X] T049 [P] [US3] Write a failing test in `src/egui_canvas/tests.rs` asserting `blueprint_preview_for_hover` returns, for each node, its `screen_rect` alongside that node's `buildable_id` and `rotation` (not a bare `Rect`).
- [X] T050 [US3] Introduce `pub(super) struct BlueprintPreviewNode { screen_rect: Rect, buildable_id: BuildableId, rotation: Rotation }` and change `blueprint_preview_for_hover`'s return type from `Vec<Rect>` to `Vec<BlueprintPreviewNode>` in `src/egui_canvas/geometry.rs` (data-model.md's breaking-change section). Update its one call site in `src/egui_canvas/mod.rs`'s `CanvasPaintLayer::Preview` arm in the **same commit** (this project's documented compile-coupling rule). Makes T049 pass.
- [X] T051 [US3] In the `CanvasPaintLayer::Preview` arm of `src/egui_canvas/mod.rs`, paint each `BlueprintPreviewNode`'s own icon (via `layout.catalog().buildable(&node.buildable_id)` and `icons.texture(...)`) or fallback text, translucent, at `node.rotation`'s resting (non-animated) orientation, alongside the existing translucent rect painting.
- [X] T052 [US3] In `src/egui_canvas/mod.rs`'s same `CanvasPaintLayer::Preview` arm, extend the single-buildable placement-preview branch to paint the armed buildable's own icon or fallback text (translucent, at rest orientation, no rotation applied since a fresh placement always starts at `Rotation::Zero`) inside the existing preview rect.
- [X] T053 [P] [US3] Extend `src/egui_canvas/tests.rs`'s existing `rejected_rotation_starts_no_transition_and_changes_nothing` test (or add a sibling) confirming `RotationVisuals`'s own contract is byte-for-byte unchanged by T034's painting changes — a rejected rotation still starts no transition, for both an icon-bearing and a text-only instance's underlying angle/position state.
- [X] T054 [US3] Run the full scoped test set (no single file — whole-binary test gate; T036/T045 plus T047-T053's new tests), confirming everything passes, including every pre-existing Phase 8 rotation-animation test unchanged.
- [ ] T055 [US3] Manual validation: Diogo runs quickstart.md Scenarios 5 and 6 end-to-end (all four presentation surfaces, accepted/rejected/group rotation, rapid turns, layout restoration).

**Checkpoint**: All three functional user stories work together — icons
appear consistently on instances, both preview kinds, and the palette;
rotation turns the correct representation smoothly with no arrow left
anywhere in the codebase or on screen.

---

## Phase 6: User Story 4 (P2) — Customize icons and machine data using the README

**Goal**: A reader unfamiliar with this feature can, using only the
README, assign an icon, change display data, and restore text-only
rendering.

**Independent Test**: Starting from the documented public sample, follow
the README to prepare a user-maintained catalog, assign an icon to the
Xiranite Power Pole, change its display name, restart, then remove the
icon reference to restore text (quickstart.md Scenario 7).

- [ ] T056 [US4] Add a new subsection to `README.md`'s existing "Runtime catalog" section (research.md Decision 8) documenting: `assets/icons/`'s location and purpose; the optional `icon` field's exact JSON shape using the existing Xiranite Power Pole example; the supported format (static PNG, including the APNG-first-frame-only caveat from research.md Decision 4); that `catalog/public/`-embedded icon changes need a rebuild while `data/catalog/`-associated icon changes only need a restart (mirroring the README's already-existing catalog-data sentence); and a short troubleshooting list (wrong/missing file, path outside `assets/icons/`, oversized file) mapped to each one's specific fallback behavior.
- [ ] T057 [US4] Manual validation: Diogo (or a reader with no prior conversation context) follows only the new README section end-to-end per quickstart.md Scenario 7.

**Checkpoint**: All four user stories are independently functional and documented.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Keep this project's other documentation in sync and close the feature with its established gates.

- [ ] T058 [P] Update `docs/data-model.md`'s buildable-fields list (the `id`, `display_name`, `category`, `symbol`, `footprint`, `production_targets` bullet list) to add the new optional `icon` field, per this project's existing convention of keeping that document in sync with the real catalog schema.
- [ ] T059 [P] Add a new "Phase 9 — custom buildable icons" entry to `docs/roadmap.md`, following the existing per-phase documentation convention (integrated-state summary, scope boundary, merge-workflow note per Constitution v1.3.0's reinstated hosted-PR requirement).
- [ ] T060 Run the complete gate sequence (no single file — whole-repository gate) across the full feature diff: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, the scoped test commands this feature actually touches (not a blanket `cargo test`), `cargo build --release --bins`, `git diff --check`, `hermes verify --skip-start --json --timeout 300`.
- [ ] T061 Open a hosted GitHub pull request for this feature branch (no single file — repository-level Git/GitHub action) per Constitution v1.3.0's reinstated PR requirement, and request an independent review per the `requesting-code-review` skill before merging with `git merge --ff-only` from a `master` confirmed equal to `origin/master`.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories. T016 is an internal gate within this phase (call-site updates must compile clean before catalog-loader work begins).
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion.
  - US1 and US2 are both P1 and share the same underlying `BuildableIcons::load` code path (US2 exercises US1's failure branches) — implement US1 first (T028-T037), then US2 (T038-T046) which mostly adds test coverage and small fixes to what US1 already built.
  - US3 (T047-T055) depends on US1's `BuildableIcons`/icon-or-text painting helper existing (T029-T034) but is otherwise independent of US2.
  - US4 (T056-T057) is documentation-only and can start any time after Foundational, though writing it after US1-US3 land means the documented example is verified against real behavior rather than planned behavior.
- **Polish (Phase 7)**: Depends on all four user stories being complete.

### Within Each User Story

- Each RED test task immediately precedes the GREEN implementation task that satisfies it (vertical tracer bullets, not a separate test-writing phase).
- T016 and T021 are internal gates — do not start Phase 3 work until both pass.

### Parallel Opportunities

- T001 and T002 (Setup) can run in parallel.
- T005-T015 (11 call-site-update tasks across 10 files) can all run in parallel once T004 is complete — different files, same mechanical change.
- T022, T023, T025 (path-safety and size-cap tests) can be written in parallel; T017 and T020 (catalog-loader tests) can be written in parallel with them too, since they touch different files.
- T028 and T033 can be written in parallel (different pure helpers, different files/concerns).
- T038, T040, T042, T044 (US2's four failure-mode tests) can all be written in parallel.
- T047 and T049 (US3's two independent surfaces — palette vs. blueprint preview) can proceed in parallel.
- T058 and T059 (Polish docs) can run in parallel.

---

## Parallel Example: Foundational call-site updates

```text
Task: "Update BuildableDefinition::new call site in tests/support/mod.rs"
Task: "Update 7 BuildableDefinition::new call sites in tests/domain_runtime_catalog.rs"
Task: "Update 2 direct BuildableDefinition::new call sites in tests/domain_blueprint.rs"
Task: "Update BuildableDefinition::new call site in tests/domain_production_target.rs"
Task: "Update BuildableDefinition::new call site in tests/factory_document_codec.rs"
Task: "Update BuildableDefinition::new call site in tests/blueprint_document_codec.rs"
Task: "Update BuildableDefinition::new call site in tests/blueprint_library.rs"
Task: "Update BuildableDefinition::new call site in src/blueprint_library_view.rs"
Task: "Update BuildableDefinition::new call site in src/history.rs"
Task: "Update 4 BuildableDefinition::new call sites in src/egui_app/tests.rs"
Task: "Update BuildableDefinition::new call site in src/egui_canvas/tests.rs"
```

All eleven touch different files and share the identical mechanical
change (add `None` as the new trailing argument), so they are safe to
parallelize — but all must land in the same commit as T004 per this
project's compile-coupling rule, since the crate does not build with
the signature changed but call sites unfixed.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories; includes the 22-call-site scope correction).
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: run quickstart.md Scenario 1 (T037).
5. This is a demonstrable MVP: one buildable's icon shows on the canvas.

### Incremental Delivery

1. Setup + Foundational → foundation ready, nothing user-visible yet.
2. + User Story 1 → an icon renders on placed instances (MVP!).
3. + User Story 2 → every failure mode is proven safe; catalogs without any icon remain fully unaffected.
4. + User Story 3 → icons (and rotated fallback text) appear consistently everywhere spec.md requires, with the arrow fully removed.
5. + User Story 4 → the feature is self-documented and independently reproducible from the README alone.
6. + Polish → cross-cutting docs updated, full gate sequence green, PR opened per Constitution v1.3.0.

Each story adds value without breaking the previous one — no story's
tasks modify a file exclusively owned by an earlier story's still-open
work, aside from the two explicitly-flagged shared-file exceptions
(`src/egui_canvas/painting.rs`'s `paint_instances`, touched by both US1
and read by US3's preview work; `src/egui_canvas/mod.rs`'s `show`,
touched by both US1's signature widening and US3's preview-arm changes).

---

## Notes

- [P] tasks = different files, no dependency on an incomplete task.
- [Story] label maps every user-story-phase task to US1/US2/US3/US4 for traceability back to spec.md.
- Every clarified requirement (spec.md's three `## Clarifications` entries — FR-003's four surfaces, FR-008's directory shape, FR-009's arrow removal) has at least one task that asserts its specific resolved value, not merely a task that implements the surrounding feature, per this project's `github-spec-kit` skill discipline: FR-003 → T033/T034 (instances), T047 (palette), T049 (previews); FR-008 → T002 (the directory itself is the deliverable, not a runtime-testable behavior); FR-009 → T034 (arrow deleted, replaced) plus T053 (rejection contract still holds).
- Commit after each task or small logical group, narrated, per this project's established `docs/engineering-standards.md` §Git convention — never silent.
- Stop at any checkpoint to validate a story independently before continuing.
- Avoid: vague tasks, same-file conflicts inside a marked-[P] group, and cross-story dependencies that would break independent testability.
