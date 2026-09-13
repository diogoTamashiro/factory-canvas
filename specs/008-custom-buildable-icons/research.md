# Research: Custom Buildable Icons

**Branch**: `008-custom-buildable-icons` | **Date**: 2026-09-13
**Spec**: [spec.md](./spec.md)

All decisions below were resolved by reading the actually-installed crate
source at the pinned lockfile versions (paths under
`~/.cargo/registry/src/index.crates.io-*/`), not from web documentation,
which can drift from the exact version this project builds against.
Every existing symbol referenced (`RotationVisuals`, `CanvasState`,
`catalog_loader`'s path-safety helpers, etc.) is quoted from this
project's own current source, read fresh this session.

## Decision 1: Icon storage location — `assets/icons/`, mirroring `catalog/public/`

**Decision**: A new top-level `assets/icons/` directory, versioned with
the project exactly like `catalog/public/` already is. Buildable JSON
gains an optional `icon` field holding a path relative to this
directory (e.g. `"icon": "xiranite_power_pole.png"`). No embedding at
build time (unlike the public catalog's `include_str!` pattern) — icons
are read from disk at startup, the same way `data/catalog/` already is.

**Rationale**: FR-008 (clarified) requires "one dedicated, documented,
versioned image directory inside the existing project... MUST NOT
require a separate image repository, submodule, ... or network setup."
`assets/` is a conventional Rust-ecosystem name for exactly this
(confirmed empirically: `eframe-0.36.1`'s own doctests reference
`egui::include_image!("../assets/ferris.png")` at
`egui-0.36.1/src/painter.rs:441` and `egui-0.36.1/src/widgets/image.rs:361`
— the crate's own convention is `assets/<name>.png`). Embedding via
`include_str!`/`include_bytes!` would force a rebuild for every icon
change, directly contradicting spec.md FR-007 ("apply the changes by
restarting without rebuilding the application").

**Alternatives considered**:
- `catalog/icons/` (nested under the existing public-catalog directory):
  rejected — `catalog/public/` is specifically the *embedded* minimal
  fallback (see `directory_source.rs`'s `EmbeddedPublicSource`, which
  `include_str!`s every file in it). Icons must NOT be embedded, so
  putting them there would blur that already-established boundary.
- A private-only `data/icons/` (parallel to the gitignored `data/catalog/`):
  rejected as the *sole* location — spec.md FR-016 requires the feature
  to work with public, redistributable example assets too, so the
  primary home must be a tracked, versioned directory. A private
  sub-path is still allowed as a per-user override (Decision 6 below).

## Decision 2: Domain change — `BuildableDefinition` gains `Option<Arc<str>>` icon field

**Decision**: Add one new field to the existing
`src/domain/catalog.rs::BuildableDefinition` struct:

```rust
pub struct BuildableDefinition {
    id: BuildableId,
    display_name: Arc<str>,
    category_id: CategoryId,
    symbol: Arc<str>,
    footprint: GridSize,
    production_targets: Vec<ProductId>,
    icon: Option<Arc<str>>,   // NEW — relative path under assets/icons/, or None
}
```

with a new accessor `pub fn icon(&self) -> Option<&str>` alongside the
existing accessors, and `BuildableDefinition::new(...)` gaining one more
parameter. This mirrors the existing style exactly — every other field
on this struct follows the identical `Arc<str>` + accessor pattern (see
current `src/domain/catalog.rs:164-215`, read this session).

**Rationale**: FR-001 requires the association to live in "that
buildable's existing JSON catalog object... without a second per-machine
mapping." `Catalog::new(...)`'s existing validation function (lines
375-472, read this session) already loops over every buildable checking
`display_name`/`symbol`; the icon field slots into the same validation
pass with zero new infrastructure. `icon` stores a *relative path
string*, not a loaded image or a `TextureHandle` — the domain layer
(per `docs/engineering-standards.md` and this project's own
`interactive-canvas-ui`/`rust-egui-desktop-app` skill rule "Keep the
domain free of egui") must not depend on egui or perform any filesystem
I/O. Actually resolving the path to bytes/pixels is entirely a
presentation-layer (`egui_app`/`egui_canvas`) concern, exactly the same
split the project already uses for `symbol` (domain stores the string,
`egui_canvas::painting::block_visual` decides what to paint with it).

**Alternatives considered**:
- A `HashMap<BuildableId, PathBuf>` icon registry separate from
  `BuildableDefinition`: rejected — this is the "second per-machine
  mapping" FR-001 explicitly rules out, and it would desynchronize from
  the buildable list whenever catalog buildables are added/removed.
- Storing a loaded `image::DynamicImage` or `egui::TextureHandle` on the
  domain struct: rejected outright — violates "domain free of egui" and
  makes `BuildableDefinition` (currently `Clone + PartialEq + Eq`, used
  in tests via simple value equality) depend on GPU-resident state that
  cannot be meaningfully compared or cloned cheaply.

### Catalog JSON contract addition

`catalog/public/buildables.json`'s existing shape (verified this
session, `catalog/public/buildables.json:1-37`) is:

```json
{
  "buildables": [
    {
      "id": "xiranite_power_pole",
      "display_name": "Xiranite Power Pole",
      "category": "energy",
      "symbol": "XPP",
      "footprint": { "width": 2, "height": 2 },
      "production_targets": []
    }
  ]
}
```

New optional field, added the same way every other field already works
— `src/catalog_loader/dto.rs`'s `BuildableDto` currently has
`#[serde(deny_unknown_fields)]` with all-required fields (verified this
session, `dto.rs:58-67`). Add:

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BuildableDto {
    pub(super) id: String,
    pub(super) display_name: String,
    pub(super) category: String,
    pub(super) symbol: String,
    pub(super) footprint: DimensionsDto,
    pub(super) production_targets: Vec<String>,
    #[serde(default)]
    pub(super) icon: Option<String>,   // NEW, optional via #[serde(default)]
}
```

`#[serde(default)]` on an `Option<String>` field means an absent `icon`
key deserializes to `None` — confirmed against `serde`'s documented
behavior for `Option<T>` fields (every other field in this DTO is
non-optional and has no `#[serde(default)]`, so this is a deliberate,
minimal addition, not a pattern change). `null` also deserializes to
`None` (serde's default `Option<T>` `Deserialize` impl accepts both a
missing key and an explicit `null`). An empty string `""` deserializes
to `Some(String::new())` — parsing code (not the DTO) must treat that
as "no icon" per spec.md's clarified FR-004/FR-012 ("Omitted, null, or
blank image references... produce no warning"), the same way
`validate_module_path` already treats an empty *catalog path* string as
invalid input requiring its own check rather than relying on serde.
`deny_unknown_fields` is untouched — a genuinely malformed `icon` value
(e.g. a JSON number or object) still fails deserialization exactly like
every other wrongly-typed field today, satisfying spec.md's requirement
that "malformed JSON, wrong field types... retain the established strict
validation."

## Decision 3: Icon resolution and safe file loading — reuse `catalog_loader`'s path-safety pattern

**Decision**: A new small module, `src/egui_app/icons.rs`
(`egui_app`-scoped, not `catalog_loader`-scoped, since it is a
presentation concern — see Decision 2's domain/presentation split), with
one pure function:

```rust
pub(super) fn resolve_icon_path(
    icons_root: &Path,       // canonicalized once at startup
    relative: &str,          // BuildableDefinition::icon()'s value
) -> Result<PathBuf, IconPathError>
```

implementing the *exact same* validation `catalog_loader`'s
`validate_module_path` (parsing.rs:20-51, read this session) and
`ensure_module_within_root` (directory_source.rs:43-52, read this
session) already do for catalog module paths: reject empty strings, NUL
bytes, a leading `/` after normalizing `\` to `/`, any `:` (Windows
drive prefix), and any `.`/`..` path component — then `fs::canonicalize`
the joined path and confirm it `.starts_with(&icons_root)` (also already
canonicalized), which is what actually defeats a symlink/junction that
points outside the intended directory: `canonicalize` resolves
symlinks/junctions to their real target before the `starts_with` check
runs, so a junction inside `assets/icons/` that points elsewhere is
caught at that comparison, not left as an unresolved shortcut. This is
not a hypothetical concern for this exact codebase — it is the
identical technique already protecting catalog module paths in
production, being extended to a second, structurally identical
"user-relative-path-into-a-known-root" problem.

**Rationale**: spec.md's clarified edge cases require rejecting
"absolute location, a parent directory, a network resource, or an
indirectly linked file outside the allowed collection" (User Story 2
scenario 6) without ever reading that external resource. Re-deriving
this from scratch would risk a subtly different (and possibly weaker)
check than the one already reviewed and shipped for catalog paths;
reusing the identical algorithm means the exact same TDD test matrix
`catalog_loading.rs`'s existing symlink tests already cover (confirmed
present at `tests/catalog_loading.rs:55-63`, using
`std::os::windows::fs::symlink_file` guarded by the project's own
documented `ERROR_PRIVILEGE_NOT_HELD` skip pattern) can be copied with
buildable-icon fixtures instead of catalog-module fixtures.

**Honest threat-model boundary** (per this project's
`conscious-orchestration` policy of reporting evidence including its
limits, and per `rust-egui-desktop-app`'s own documented Windows
filesystem gotchas): `canonicalize`-then-`starts_with` is a
point-in-time check, not a sandboxed read — between canonicalizing and
actually opening the file there is a narrow TOCTOU window on a
filesystem another process could mutate concurrently (junction
retargeted, file replaced). This is the same class of limitation the
project's catalog loader already accepts (it performs the identical
canonicalize-then-check-then-read sequence, not an atomically-locked
read), and matches this project's already-declared trust model: local,
single-user, offline-only data the *user themselves* maintains — not
data received over a network or from a mutually-untrusted process.
Extending icon loading to the identical model does not introduce a new
risk class beyond what catalog loading already accepts today.

**Resource caps** (new, since icons are new binary — not JSON — input):
cap the *raw file byte length* before decoding (proposed: 4 MiB per
icon — generous for a small UI icon, tiny relative to
`image::io::Limits::default()`'s already-permitted 512 MiB decoded
allocation ceiling, confirmed at
`image-0.25.10/src/io/limits.rs:49-56`, read this session). Checking the
raw file length first, before calling any decoder, avoids ever handing
a maliciously-huge file to the PNG decoder just to have it reject a
derived limit afterward — cheap `fs::metadata` first, decode only files
that already pass. This is a new, feature-specific decision (not a
reuse of an existing project pattern), since catalog JSON has no
equivalent "this could be a huge binary blob" concern today.

## Decision 4: PNG decoding — `image` 0.25.10, already locked, promote from transitive to direct dependency

**Decision**: Add `image = { version = "0.25.10", default-features =
false, features = ["png"] }` as a new **direct** dependency in
`Cargo.toml`, pinned to the exact version already resolved. Decode with:

```rust
let dynamic_image = image::load_from_memory_with_format(
    &file_bytes,
    image::ImageFormat::Png,
)?;   // rejects non-PNG bytes regardless of the file's extension
let rgba = dynamic_image.to_rgba8();
let size = [dynamic_image.width() as usize, dynamic_image.height() as usize];
let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
```

**Rationale — this adds ZERO new transitive dependencies**: `cargo tree
-i image@0.25.10` (run this session against this project's real,
already-resolved `Cargo.lock`) shows `image v0.25.10` is *already*
pulled in twice — by `eframe v0.36.1` itself (`Cargo.lock`, `eframe`'s
own `[target...dependencies.image]` block pins `version = "0.25.6"`,
`features = ["png"]`, `default-features = false`, confirmed at
`eframe-0.36.1/Cargo.toml:257-260`, read this session) and by
`arboard v3.6.1` (eframe's clipboard-image dependency). It is compiled
into this project's dependency tree on every build today regardless of
this feature. The *only* change `Cargo.toml` needs is declaring it as a
direct dependency so `factory-canvas`'s own source can `use image::...`
— Rust/Cargo do not let a crate use a transitive dependency's public API
without its own `Cargo.toml` entry, even though the crate is already
being compiled. `features = ["png"]` matches exactly what `eframe`
itself already requests, so no new codec, no new feature flag
surface, and `Cargo.lock`'s resolved version does not change (`0.25.10`
already satisfies eframe's own `"0.25.6"` requirement, confirmed via the
existing lockfile).

**Why `load_from_memory_with_format` over `load_from_memory` or
extension-based dispatch**: `load_from_memory` (verified at
`image-0.25.10/src/images/dynimage.rs:1676-1680`, read this session)
"makes an educated guess about the image format" from magic bytes via
`ImageReader::with_guessed_format`. That guess is still content-based
(not extension-based), so it is not unsafe per se, but this project only
wants to support PNG (spec.md FR-014, "MUST support static PNG icons...
Other image formats MUST use the same safe fallback"). Calling
`load_from_memory_with_format(bytes, ImageFormat::Png)` instead means a
file that is a renamed JPEG/BMP/etc with a `.png` extension fails to
decode (wrong magic bytes for the declared format) rather than silently
succeeding as some other format — this is what "supports PNG, other
formats fall back safely" actually requires: reject by *content*, not
merely accept by extension.

**PNG feature scope actually decoded** (confirmed from the vendored
`png` 0.18.1 crate that `image`'s `png` feature depends on, `Cargo.lock`
entry read this session): `png::ColorType` supports
`Grayscale`/`Rgb`/`Indexed`/`GrayscaleAlpha`/`Rgba` (verified at
`png-0.18.1/src/common.rs:12-20`) at both 8-bit and 16-bit depth — `image`'s
`to_rgba8()` normalizes any of these (grayscale, palette/indexed,
with or without alpha, 8- or 16-bit) to a flat 8-bit RGBA buffer
uniformly, so no per-PNG-subtype branching is needed in this project's
own code. Animated PNG (APNG): `image`'s PNG decoder returns only the
first/default frame through the standard `ImageDecoder`/`DynamicImage`
path used here (this project never calls image's separate
`AnimationDecoder` trait), so an APNG file is read as a single static
image — satisfying spec.md's "Other image formats MUST use the same
safe fallback" only insofar as animation is silently ignored, not
rejected; this is an accepted, minor scope note; document PNG support
as "static PNG (a multi-frame APNG's first frame only)" in the README
rather than silently promising animation support.

**Alternatives considered**:
- Writing a hand-rolled minimal PNG decoder: rejected outright — this
  project's engineering standards (`docs/engineering-standards.md`
  §Dependencies: "prefer `std`... every crate needs a documented current
  benefit") do not mandate zero dependencies, they mandate no
  *unjustified* ones; here the dependency is already fully paid for by
  `eframe` itself, so writing custom decode logic would be strictly
  worse: more code to maintain, a real security surface (PNG parsing is
  exactly the kind of format-parsing code advisories exist for), and it
  would reject the "prefer `std`, but a crate needs a documented current
  benefit" cost/benefit the standard actually asks for.
- Depending on `png` 0.18.1 directly instead of `image`: rejected —
  would require this project's own code to implement palette-to-RGBA,
  16-bit-to-8-bit, and grayscale-to-RGBA conversion by hand (`png`'s API
  is a lower-level streaming decoder, not a "give me a `DynamicImage`"
  convenience layer); `image` already provides exactly the uniform
  `to_rgba8()` this project needs, at no extra dependency cost since
  `image` (with the `png` feature) is already compiled in.
- `egui_extras`'s built-in image loaders (`egui_extras::install_image_loaders`,
  referenced only in doc comments this session, not currently a
  dependency of this project — confirmed absent from `Cargo.lock` and
  `Cargo.toml`): rejected for this increment — that mechanism targets
  `egui::Image::from_uri(...)`'s async/`include_image!` loader
  registration flow (bytes/file/HTTP loaders keyed by URI scheme), which
  is a heavier general-purpose abstraction than this feature needs; this
  project instead decodes eagerly (icons are local, already-validated
  files, not remote/async resources) and constructs `TextureHandle`s
  directly via `Context::load_texture`, matching the "no live reload,
  no async" boundary spec.md already sets (FR-007).

## Decision 5: Icon cache lifecycle — app-owned `BuildableIcons`, populated once in `FactoryCanvasApp::new`

**Decision**: A new struct in `src/egui_app/icons.rs`:

```rust
pub(super) struct BuildableIcons {
    textures: std::collections::BTreeMap<BuildableId, egui::TextureHandle>,
    warnings: Vec<String>,   // sanitized, per spec.md FR-004
}
```

populated by one function called only from
`FactoryCanvasApp::new(creation_context)` (`src/egui_app/startup.rs:102-110`,
read this session — the ONLY call site that already has a live
`egui::Context` via `creation_context.egui_ctx`), iterating
`catalog.buildables()`, resolving+decoding each buildable's `icon()` per
Decisions 3-4, and calling
`creation_context.egui_ctx.load_texture(name, color_image, TextureOptions::LINEAR)`
(verified signature at `egui-0.36.1/src/context.rs:2387-2392`, read this
session) for each successfully-decoded icon. A per-icon decode/load
failure appends a sanitized warning to `warnings` and simply omits that
buildable from `textures` (BTreeMap lookup miss = "no icon" at every
paint site, reusing Decision 2's `Option`-shaped fallback contract
uniformly) — it does not fail catalog loading, matching spec.md FR-004's
"resolve independently to text rather than prevent editing... or
replace otherwise-valid catalog data."

`FactoryCanvasApp` gains one new field, `icons: BuildableIcons`, next to
the existing `catalog_warning: Option<String>` field
(`src/egui_app/mod.rs:36`, read this session) — `icons.warnings` folds
into the existing notice/warning surface the same way `catalog_warning`
already does, rather than inventing a second warning channel.

**Rationale — why `new()` only, not `from_startup_catalog`/`Default`**:
`from_startup_catalog` (`startup.rs:80-100`) and `impl Default for
FactoryCanvasApp` (`mod.rs:49-58`) are both called from test code
extensively (confirmed: dozens of call sites across
`src/egui_app/tests.rs`, e.g. `startup_test_catalog(...)` helpers) and
have no live `egui::Context` available to call `load_texture` against —
`CreationContext` is only constructed by real `eframe::run_native`
startup. Keeping icon-texture loading exclusively in `new()` means
every existing test that builds an app via `from_startup_catalog`/
`Default` continues to run with zero filesystem or GPU-texture I/O,
exactly as today — this satisfies this project's own documented rule
(`rust-egui-desktop-app` skill, "Scope `cargo test`... ") of keeping the
domain/test-constructible surface IO-free, extended here to the new
icon surface. `BuildableIcons::empty()` (no textures, no warnings) is
the `Default` value used by test-only construction paths, matching how
`catalog_warning: None` is already the test-default for the analogous
existing field.

**Cache keying and invalidation**: keyed by `BuildableId` (the stable
catalog identity, not the icon's file path) because a catalog reload —
which does not happen mid-session per spec.md's assumptions ("no live
reload... users close the application before editing and restart
afterward") — is the only thing that would need this cache invalidated,
and it already only happens at the next full-process restart (a fresh
`BuildableIcons` is built from scratch on the next `new()` call). No
cache-invalidation logic is needed within a single running session.

## Decision 6: Rendering — reuse `RotationVisuals` exactly as-is; paint via `egui::Image::rotate`/`epaint::TextShape.angle`

**Decision**: `egui_canvas::rotation::RotationVisuals` (already
implemented, `src/egui_canvas/rotation.rs`, resync'd from all four
whole-layout-replacement call sites per this project's own
Phase-8-established contract) is used completely unmodified — it
already exposes `visual_state_for(ctx, id, rotation, origin) -> (angle,
origin_x, origin_y)` (confirmed still present and unchanged at
`src/egui_canvas/painting.rs:136-141`, read this session). The single
change needed is in `paint_instances`
(`src/egui_canvas/painting.rs:121-173`, read this session): replace the
existing `paint_orientation_arrow(painter, screen_rect.center(), ...)`
call (line 169) with a branch on whether the resolved buildable has a
usable icon.

**With an icon** (spec.md FR-009, clarified: "rotate the image
smoothly"):

```rust
let angle_radians = angle.to_radians();   // `angle` is already in
                                           // degrees per this project's
                                           // existing convention, see
                                           // rotation_degrees()
egui::Image::from_texture(&texture_handle)
    .rotate(angle_radians, egui::Vec2::splat(0.5))
    .paint_at(ui, screen_rect);
```

`Image::rotate(angle, origin)` (verified at
`egui-0.36.1/src/widgets/image.rs:228-240`, read this session): "Rotate
the image about an origin by some angle. Positive angle is clockwise.
Origin is a vector in normalized UV space... To rotate about the center
you can pass `Vec2::splat(0.5)`." This matches
`RotationVisuals`/`Rotation`'s own already-documented clockwise
convention exactly (this project's existing
`orientation_arrow_direction` comment in `painting.rs` explicitly notes
"matching `Rotation`'s own documented clockwise direction" — same
convention, now reused for the image path instead of the arrow path).
Internally this calls `paint_texture_at` (verified at
`egui-0.36.1/src/widgets/image.rs:839-874`, read this session), which
for the rotated branch builds a real `Mesh` via
`Mesh::add_rect_with_uv` + `Mesh::rotate(rot, pivot)` (confirmed method
exists at `epaint-0.36.1/src/mesh.rs`, read this session) — genuine
per-frame mesh rotation of the actual texture, not a pre-rasterized
rotated bitmap; no new state, no interpolation logic beyond what
`RotationVisuals` already produces every frame.

Note: `Image::rotate`'s own doc comment states "Due to limitations in
the current implementation, this will turn off rounding of the image"
(`image.rs:235-236`) — since this project's icon footprint tiles are
drawn with `painter.rect_filled(screen_rect, 2, fill)`'s *background*
rectangle already providing the visible corner rounding (2px, per
`painting.rs:151`), the icon image itself does not need its own corner
rounding — it paints inside that already-rounded background rect, so
this limitation does not affect the visible result.

**Without an icon (fallback text)** (spec.md FR-009, clarified: "rotate
the fallback text smoothly instead"):

```rust
let galley = painter.layout_no_wrap(
    label.to_owned(),
    FontId::proportional((screen_rect.height() * 0.4).clamp(8.0, 11.0)),
    TEXT_PRIMARY,
);
let text_shape = epaint::TextShape {
    pos: screen_rect.center() - galley.size() * 0.5 /* pre-rotation centering */,
    angle: angle_radians,
    ..epaint::TextShape::new(pos, galley, TEXT_PRIMARY)
};
painter.add(egui::Shape::Text(text_shape));
```

`TextShape.angle` (verified at
`epaint-0.36.1/src/shapes/text_shape.rs:39-41`, read this session):
"Rotate text by this many radians clockwise. The pivot is `pos` (the
upper left corner of the text)." Since the pivot is the *upper-left*
corner (not center, unlike `Image::rotate`'s configurable origin), the
existing centered-text call this project currently uses
(`painter.text(screen_rect.center(), Align2::CENTER_CENTER, label,
...)`, `painting.rs:161-167`) cannot be reused as-is — `painter.text(...)`
(verified at `egui-0.36.1/src/painter.rs:469-481`, read this session) has
no angle parameter at all, it is a convenience wrapper that always calls
`self.galley(rect.min, galley, text_color)` with zero rotation. The
replacement must lay out the galley once (`painter.layout_no_wrap`,
confirmed at `painter.rs:503-510`), compute its pre-rotation
top-left-if-centered position manually
(`screen_rect.center() - galley.size() * 0.5`, standard "un-apply
`Align2::CENTER_CENTER`" arithmetic), build a `TextShape` with that
`pos` and the desired `angle`, and paint it via `painter.add(Shape::Text(...))`
instead of the no-angle `painter.text(...)` convenience method. This is
a NEW code path (this project's existing text painting never rotates),
not a reuse of an existing helper — flagged explicitly here per this
research document's own labeling convention.

**Why this needs no new animation/interpolation state**: both branches
consume the *same* `angle` value `RotationVisuals::visual_state_for`
already computes and animates every frame (the existing `egui::Context::
animate_value_with_time`-based interpolation this project's Phase 8
already implemented, including its four documented gotchas in the
`rust-egui-desktop-app` skill) — the only change is which shape that
angle gets applied to (a rotated image mesh, or a rotated text galley)
instead of the removed arrow triangle. `paint_orientation_arrow` and
`orientation_arrow_direction` (the arrow-specific helpers,
`painting.rs:72-93`) are deleted entirely per spec.md FR-009's "The
placeholder orientation arrow MUST be removed completely" — nothing
about `RotationVisuals` itself, `CanvasState`, or the animation
machinery changes.

**Alternatives considered**:
- Pre-rendering rotated bitmap variants (e.g. 4 pre-rotated PNGs per
  icon, or rasterizing rotated frames into a texture atlas): rejected —
  the transition is continuous/animated (spec.md FR-009 requires smooth
  turning, not 90°-stepped snapping), so any fixed set of pre-rotated
  bitmaps could not represent in-between animation frames without
  either visible stepping or the same mesh-rotation approach applied to
  a texture anyway; `Image::rotate` already does real per-frame
  rotation at zero extra cost.
- Rasterizing the fallback text into a bitmap and reusing the icon
  image-rotation path uniformly for both cases: rejected — this was
  explicitly ruled out by the spec's own research question ("without
  rasterizing text into images") and by `TextShape.angle` already
  existing as a first-class, already-antialiased vector-text rotation
  primitive; rasterizing would add a new runtime texture-generation step
  (one dynamic texture per distinct rotated label) for strictly worse
  visual quality (blurry rescaled bitmap text vs. crisp vector glyphs)
  and non-trivial new cache-invalidation complexity this project does
  not otherwise need.

## Decision 7: Palette/preview integration — extend existing call sites, not new parallel ones

**Decision**: Four existing call sites gain icon awareness, matching
spec.md's four required surfaces (FR-003) exactly:

1. **Placed instances** — `egui_canvas::painting::paint_instances`
   (Decision 6, above). Requires `paint_instances` to receive a new
   `&BuildableIcons`-shaped lookup parameter (a small trait or plain
   `&BTreeMap<BuildableId, TextureHandle>` reference — final shape
   decided in `data-model.md`/`plan.md`, not this research document).

2. **Single-buildable placement preview** — the existing `Preview`
   paint-layer branch in `egui_canvas::mod.rs::show()`
   (`mod.rs:272-289`, read this session), which currently paints only a
   translucent filled+stroked rect via `placement_preview_visual(...)`
   with no label/icon at all today. Add the same
   icon-or-rotated-text-at-normal-orientation painting Decision 6
   describes, at `preview.rotation`'s stored (non-animated — spec.md
   FR-017 explicitly excludes preview from the transition system)
   angle, with reduced opacity to preserve the existing "translucent,
   does not imply acceptance" semantic (`docs/roadmap.md`'s already
   established "The translucent preview is visual only" contract).

3. **Blueprint-member preview** — `blueprint_preview_for_hover`
   (`src/egui_canvas/geometry.rs:77-104`, read this session) currently
   returns `Vec<Rect>` only (screen rectangles, no buildable identity
   surviving past the `.filter_map`). This must change to return enough
   information to paint each member's own icon —
   `Vec<(Rect, BuildableId, Rotation)>` or an equivalent small struct
   (exact shape is a `data-model.md` decision, not resolved here) — a
   **breaking signature change** to an existing `pub(super)` function
   and its one call site in `mod.rs:239-245`. This is the only genuinely
   new plumbing this feature requires beyond icon storage/loading/paint
   itself; flag it explicitly in `plan.md`'s task breakdown as
   higher-risk than the other three (mechanically simple) surfaces since
   it touches an existing function's return type and its test
   expectations (`egui_canvas/tests.rs` — confirmed this project's
   established rule from `rust-egui-desktop-app`'s skill entry
   "Splitting a feature into layered commits: check compile coupling
   first" applies: this signature change and its call site must land in
   the same commit).

4. **Buildable palette** — `block_palette_ui`
   (`src/egui_app/ui/sidebar.rs:214-252`, read this session) currently
   builds each row via `Button::new(label).selected(selected)`
   (`sidebar.rs:241`). Per Decision "Swap `Label`+`Sense::click()` for
   `Button::new(...).selected(...)`" already documented in this
   project's `rust-egui-desktop-app` skill (established for the sidebar
   instance list in Phase 7), the palette already uses `Button` — adding
   an icon means switching to
   `Button::image_and_text(image, label).selected(selected)`
   (verified at `egui-0.36.1/src/widgets/button.rs:97-99`, read this
   session: "Creates a button with an image to the left of the text"),
   which routes through the *exact same* `AtomLayout`/`push_right`
   mechanism `Button::new(text_only)` already uses internally (confirmed
   at `button.rs:44-62` and `101-115`, read this session) — meaning
   `Role::Button`, `toggled()`, keyboard focusability, and text wrapping
   are preserved automatically with no new AccessKit wiring, the same
   empirical guarantee this project's skill already documents for the
   text-only case. A buildable with no icon keeps calling
   `Button::new(label)` exactly as today (spec.md FR-011's "A palette
   entry without an icon MUST remain a text-only control").

**Rationale for extending rather than duplicating call sites**: every
one of these four surfaces already has exactly one production call
site; this project's own documented engineering principle (KISS/DRY —
`docs/engineering-standards.md`) and its established pattern of a
single spatial/painting authority per concern (`FactoryLayout::place` as
sole placement authority, `RotationVisuals` as sole animation authority)
both argue against introducing a second, icon-specific painting path
that could drift from the text-fallback path's behavior over time.

## Decision 8: README documentation structure

**Decision**: Extend the README's existing "Runtime catalog" section
(`README.md:58-72`, read this session — already documents the
public/private catalog precedence table and the "add a region, base,
buildable, or product" checklist) with a new subsection, rather than a
wholly separate README section, since icons are additive data on an
already-documented buildable object, not a new independent subsystem.
Cover, in order (mapping directly to spec.md FR-015's required content
list): where `assets/icons/` lives and what it is for; the optional
`icon` field's exact JSON shape on one buildable (using the existing
Xiranite Power Pole example already in the README, per spec.md User
Story 4 scenario 2); the supported format (static PNG only, noting the
APNG-first-frame-only caveat from Decision 4); that both `catalog/public/`
edits (rebuild required, matching the README's already-existing "Changes
to `catalog/public/` require a rebuild" sentence) and `data/catalog/`
edits (restart only, matching the README's already-existing "close the
app... and restart it afterward" sentence) apply identically to icon
changes as they already do to catalog data changes; and a short
troubleshooting list (wrong/missing extension, file outside
`assets/icons/`, oversized file) each mapped to the specific fallback
behavior Decision 3/5 already define.

**Rationale**: spec.md FR-015 requires the guide to "distinguish changes
to user-maintained assets/data from changes to the bundled public
catalog that still require a rebuild" — the README already draws this
exact distinction for catalog *data*; icons inherit the identical
distinction rather than needing a new explanation invented from
scratch.

## Summary of what remains for `plan.md`/`data-model.md` (explicitly out of scope for this research document)

- The exact return-type shape for `blueprint_preview_for_hover`'s
  breaking change (Decision 7, item 3).
- The exact parameter/trait shape `paint_instances` and the palette/
  preview call sites use to reach `BuildableIcons` (a reference
  threaded as a new parameter vs. living on `CanvasState` — this
  project's own documented clippy `too_many_arguments` rule already
  argues for folding it into existing per-view state rather than a
  bare new parameter, but the concrete struct field placement is a
  design decision for `plan.md`, not this research).
- Whether `BuildableIcons` warnings surface through the existing
  `EditorNotice`/`catalog_warning` channel verbatim or need one new
  variant — `data-model.md`'s job.
- The precise TDD task breakdown and commit boundaries — `tasks.md`'s
  job (per `/speckit-tasks`, not run yet).

## Symbol labeling key (per this research's own convention, used
throughout)

- **Existing API, unmodified, reused as-is**: `RotationVisuals`,
  `Image::rotate`, `TextShape.angle`, `Button::image_and_text`,
  `Context::load_texture`, `SizedTexture::from(&TextureHandle)`,
  `image::load_from_memory_with_format`, `catalog_loader`'s
  `validate_module_path`/`ensure_module_within_root` pattern (reused by
  adaptation, not by direct call — the new module implements the same
  algorithm for a second root directory).
- **New symbols this feature introduces**: `BuildableDefinition::icon`
  field + accessor, `BuildableDto::icon`, `src/egui_app/icons.rs`
  (`BuildableIcons`, `resolve_icon_path`, `IconPathError`),
  `FactoryCanvasApp::icons` field, the new rotated-text-shape
  construction in `paint_instances`, and
  `blueprint_preview_for_hover`'s changed return type.
