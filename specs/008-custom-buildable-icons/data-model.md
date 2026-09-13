# Data Model: Custom Buildable Icons

**Branch**: `008-custom-buildable-icons` | **Date**: 2026-09-13
**Spec**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

## Domain layer (`src/domain/catalog.rs`) — one field addition

### `BuildableDefinition` (existing type, modified)

| Field | Type | Change |
|---|---|---|
| `id` | `BuildableId` | unchanged |
| `display_name` | `Arc<str>` | unchanged |
| `category_id` | `CategoryId` | unchanged |
| `symbol` | `Arc<str>` | unchanged |
| `footprint` | `GridSize` | unchanged |
| `production_targets` | `Vec<ProductId>` | unchanged |
| `icon` | `Option<Arc<str>>` | **NEW** |

New accessor: `pub fn icon(&self) -> Option<&str>`, following the exact
shape of the existing `symbol(&self) -> &str` accessor.

`BuildableDefinition::new(...)` gains one new trailing parameter:
`icon: Option<&str>` — call sites (`catalog_loader`, every test fixture
across `tests/domain_runtime_catalog.rs`, `tests/domain_block_catalog.rs`,
`src/egui_canvas/tests.rs`'s `public_buildable` helper, etc.) all need
updating in the same commit as this signature change, per this
project's own documented "check compile coupling first" rule.
`Option<&str>` was chosen over an originally-proposed
`Option<impl Into<Arc<str>>>` because the generic form cannot infer its
type parameter from a bare `None` argument (confirmed empirically
during implementation — `error[E0283]`); see `tasks.md`'s "Deviations
from plan.md" section.

**Validation**: `icon`'s *string content* (if `Some`) is NOT validated
by `Catalog::new(...)`'s existing validation pass — per Decision 2 in
research.md, path-safety validation belongs to the presentation layer
(`resolve_icon_path`, egui_app-scoped), not the domain. The domain only
stores the string exactly as decoded from JSON, mirroring how
`production_targets` stores `ProductId`s validated for existence but the
*domain* never touches a filesystem. An empty string `Some("")` is valid
domain state — the presentation layer treats it as "no icon" at paint
time, per spec.md's clarified FR-004/FR-012.

**Invariant preserved**: `BuildableDefinition` remains `Clone + PartialEq
+ Eq` with zero I/O or egui dependency — adding `Option<Arc<str>>`
preserves both traits trivially (`Arc<str>` is already `PartialEq + Eq`
for the existing `symbol`/`display_name` fields).

## Catalog loader layer (`src/catalog_loader/`) — one DTO field addition

### `BuildableDto` (existing type, `src/catalog_loader/dto.rs`, modified)

| Field | Type | Change |
|---|---|---|
| `id` | `String` | unchanged |
| `display_name` | `String` | unchanged |
| `category` | `String` | unchanged |
| `symbol` | `String` | unchanged |
| `footprint` | `DimensionsDto` | unchanged |
| `production_targets` | `Vec<String>` | unchanged |
| `icon` | `Option<String>` with `#[serde(default)]` | **NEW** |

`src/catalog_loader/mod.rs`'s buildable-mapping closure (the
`buildables.buildables.into_iter().enumerate().map(...)` block,
currently lines 161-201) passes `buildable.icon` straight through to
`BuildableDefinition::new(...)`'s new parameter — no new validation, no
new `CatalogLoadError` variant. An `icon` value that fails to
deserialize as a string (wrong JSON type) already produces the existing
`CatalogLoadError::InvalidJson { kind: CatalogJsonErrorKind::Schema,
.. }` for free, via `deny_unknown_fields`'s sibling mechanism (serde's
normal type-mismatch error), through the existing `parse_json::<T>`
wrapper — zero new error-handling code needed for that case.

## Presentation layer (`src/egui_app/icons.rs`) — new module

### `BuildableIcons` (new type)

```text
BuildableIcons
  textures: BTreeMap<BuildableId, egui::TextureHandle>
  warnings: Vec<String>          // sanitized, safe to display verbatim
```

- `BuildableIcons::empty() -> Self` — zero textures, zero warnings; the
  value every test-only app constructor (`from_startup_catalog`,
  `Default`) uses.
- `BuildableIcons::load(ctx: &egui::Context, catalog: &Catalog, icons_root: &Path) -> Self`
  — the only function that performs filesystem I/O or calls
  `Context::load_texture`; called exactly once, from
  `FactoryCanvasApp::new`.
- `BuildableIcons::texture(&self, id: &BuildableId) -> Option<&egui::TextureHandle>`
  — the sole read accessor every paint site uses. A `None` result is the
  single, uniform "fall back to text" signal for every one of the four
  presentation surfaces (spec.md FR-002/FR-004's contract expressed as
  one Rust `Option`, not four separate fallback checks).

### `IconPathError` (new type, presentation-layer-only — never seen by the domain)

```text
IconPathError
  Empty
  NulByte
  Rooted
  WindowsPrefix
  CurrentDirectory
  ParentDirectory
  OutsideRoot
  Io(std::io::ErrorKind)
  TooLarge { limit_bytes: u64, actual_bytes: u64 }
  Decode                          // image::ImageError, sanitized away
```

Mirrors `CatalogPathErrorKind`'s existing variant shape (research.md
Decision 3) plus two new variants (`TooLarge`, `Decode`) specific to
binary icon files, which catalog JSON validation has no equivalent for.
Every variant maps to a short, sanitized, path-free warning string —
same "no raw path/identifier leakage" contract `safe_catalog_load_detail`
(`src/egui_app/notices.rs`, existing) already implements for catalog
errors; `icons.rs` gets its own `safe_icon_error_detail`-shaped helper
following that precedent rather than reusing catalog's error type
directly (the variant sets are genuinely different — file-size and
decode failures have no catalog-JSON equivalent).

## `FactoryCanvasApp` (existing type, `src/egui_app/mod.rs`) — one field addition

| Field | Type | Change |
|---|---|---|
| ...14 existing fields... | | unchanged |
| `icons` | `BuildableIcons` | **NEW** |

Populated only in `FactoryCanvasApp::new(creation_context)`
(`src/egui_app/startup.rs`); every other constructor path
(`from_startup_catalog`, `impl Default`) sets it to
`BuildableIcons::empty()`. No new `Default`/`PartialEq`/`Clone` bounds
needed — `FactoryCanvasApp` itself derives none of those today (verified:
`struct FactoryCanvasApp` at `mod.rs:31` has no derive attribute at
all), so adding a non-`Clone` field (a `TextureHandle` map is not
meaningfully cloneable in the "two apps share GPU textures" sense
anyway) introduces no new constraint.

## `RotationVisuals` (existing type) — UNCHANGED

Explicitly not modified. `visual_state_for(ctx, id, rotation, origin) ->
(angle, origin_x, origin_y)` already returns everything both the
icon-rotation and text-rotation paint branches need. This data model
document records this as a deliberate non-change, not an oversight —
the entire point of research.md Decision 6 is that no new animation
state is needed.

## `blueprint_preview_for_hover` (existing function, `src/egui_canvas/geometry.rs`) — return type change

**Before**:
```rust
pub(super) fn blueprint_preview_for_hover(...) -> Vec<Rect>
```

**After**:
```rust
pub(super) struct BlueprintPreviewNode {
    pub(super) screen_rect: Rect,
    pub(super) buildable_id: BuildableId,
    pub(super) rotation: Rotation,
}

pub(super) fn blueprint_preview_for_hover(...) -> Vec<BlueprintPreviewNode>
```

A small named struct rather than a bare tuple, matching this project's
existing style for multi-field return shapes (e.g. `GridSelectionRect`,
`MarqueeFrameResult`, both plain structs with `pub(super)` fields, no
methods beyond simple constructors). `rotation` is carried so the
preview can show each member at its own saved orientation
(spec.md User Story 3 scenario 3: "every member's preview uses its own
associated icon or textual fallback at its saved relative position and
orientation") without animating it (FR-017: preview orientation is
static, not transitioned).

Call site (`src/egui_canvas/mod.rs`'s `CanvasPaintLayer::Preview` arm,
currently lines 290-303) changes from iterating bare `Rect`s to
iterating `BlueprintPreviewNode`s and resolving each node's own
`BuildableDefinition` via `layout.catalog().buildable(&node.buildable_id)`
(the same catalog lookup pattern the same function already uses one
branch above it, for the single-buildable preview at lines 274-277) to
find that buildable's icon.

## State transitions

None. This feature introduces no new mutable domain state, no new
document/history commands, and no new undo/redo entries — `icons`,
`BuildableIcons`, and the new preview struct are all either immutable
after construction (`BuildableIcons`, built once at startup) or
per-frame-recomputed read-only projections (`BlueprintPreviewNode`s,
recomputed every frame exactly like the existing `Rect`s were). This
matches spec.md's edge case: "An icon-only edit changes no factory
content: it creates no document edit or undo-history entry."

## Validation rules summary (traceability to spec.md FR IDs)

| Rule | Enforced by | FR |
|---|---|---|
| `icon` associates to the buildable type via existing JSON object | `BuildableDto.icon` + `BuildableDefinition.icon` | FR-001 |
| Usable icon replaces text; unusable/absent keeps text | `BuildableIcons::texture(...)` returning `Option` | FR-002, FR-004 |
| Icons on 4 named surfaces only | `paint_instances`, preview paint arms, `block_palette_ui` — no other call site touched | FR-003, FR-017 |
| No network/absolute/parent/symlink escape | `resolve_icon_path` (research.md Decision 3) | FR-005 |
| Contained, non-cropped, transparency-preserving at rest and during rotation | `Image::rotate`'s UV-mapped mesh paints inside `screen_rect` unchanged from today's sizing math | FR-006 |
| Add/replace/remove via restart, no rebuild | `assets/icons/` read from disk, not embedded | FR-007 |
| One dedicated versioned directory, no second repo | `assets/icons/` as a plain tracked directory | FR-008 |
| No arrow; icon or text rotates smoothly; rejected/plain-move/restore unaffected | `RotationVisuals` reused unmodified; `paint_orientation_arrow` deleted | FR-009 |
| No change to identity/footprint/validation/documents | Zero changes to `src/domain/layout.rs`, `src/persistence/`, `src/history.rs` | FR-010 |
| Palette icon supplements name; no-icon palette stays text-only | `Button::image_and_text` vs `Button::new` branch in `block_palette_ui` | FR-011 |
| Existing catalogs unchanged; malformed data still rejected | `#[serde(default)]` on `Option<String>`; `deny_unknown_fields` untouched | FR-012 |
| Documents never require icon files | No document/blueprint schema touched | FR-013 |
| Static PNG only; safe fallback otherwise | `image::load_from_memory_with_format(.., ImageFormat::Png)` | FR-014 |
| README covers icons + data customization | New README subsection (research.md Decision 8) | FR-015 |
| No official artwork required to ship | `assets/icons/` may ship empty or with redistributable samples only | FR-016 |
