# Quickstart: Custom Buildable Icons

**Branch**: `008-custom-buildable-icons` | **Date**: 2026-09-13
**Spec**: [spec.md](./spec.md) | **Data model**: [data-model.md](./data-model.md)

This is a validation guide — it proves the feature works end-to-end once
implemented. It contains no implementation code; see `tasks.md` (not yet
generated) for the actual build steps.

## Prerequisites

- A working `factory-canvas` checkout on this branch, building cleanly
  (`cargo build --bins`).
- No private `data/catalog/` package is required — every scenario below
  uses only the tracked `catalog/public/` package plus one temporary
  test PNG you create.

## Scenario 1 — Assign an icon to an existing buildable, see it on the canvas

**Proves**: FR-001, FR-002, User Story 1.

1. Create a small opaque or transparent PNG (any size — e.g. a 32×32
   solid-color square) and save it as
   `assets/icons/xiranite_power_pole.png`.
2. Edit `catalog/public/buildables.json`, adding `"icon":
   "xiranite_power_pole.png"` to the `xiranite_power_pole` entry.
3. Rebuild (`cargo build --bin factory-canvas`) — required here only
   because `catalog/public/` is embedded at compile time; a
   `data/catalog/` private package would only need a restart (Scenario
   4 below exercises that path).
4. Run the app, choose a base, place a Xiranite Power Pole.
5. **Expected**: the placed instance shows the PNG image instead of the
   `XPP` abbreviation text.
6. Place a Refinery Unit or Crushing Unit (no `icon` field set).
7. **Expected**: unaffected — still shows its abbreviated text (`RU`/`CU`).

## Scenario 2 — Missing/broken icon falls back to text, catalog stays usable

**Proves**: FR-004, FR-012, User Story 2.

1. Starting from Scenario 1's edited `buildables.json`, change the
   `icon` value to `"does_not_exist.png"` (a file that does not exist
   under `assets/icons/`).
2. Rebuild and run.
3. **Expected**: the Xiranite Power Pole instance shows its `XPP` text
   fallback (not a crash, not a missing/blank buildable). A sanitized,
   non-blocking warning is visible somewhere in the app's existing
   notice/warning surface, without echoing the file path.
4. Repeat with `icon` set to `""` (empty string).
5. **Expected**: text fallback, but this time with NO warning (an
   intentionally-omitted icon is silent — only an explicit-but-unusable
   reference warns).
6. Repeat with `icon` set to `null` explicitly in the JSON.
7. **Expected**: identical to step 5 — `null` and omission behave
   identically.

## Scenario 3 — Path safety: reject escape attempts without reading them

**Proves**: FR-005, the clarified edge case ("absolute location, parent
directory, network resource, or indirectly linked file").

1. Set `icon` to `"../../../../Windows/System32/some-real-file.png"` (a
   real file well outside `assets/icons/`, to make an accidental
   "succeeded despite the attempt" failure obvious rather than silently
   indistinguishable from "file not found").
2. Rebuild and run.
3. **Expected**: text fallback, sanitized warning — behaves exactly like
   Scenario 2's missing-file case from the outside. (Verifying the
   *attempt itself* was rejected before any read, rather than merely
   that some read failed, is covered by an automated test in
   `tests/`, not by this manual scenario alone — this scenario proves
   the user-visible outcome; `tasks.md` will pair it with a
   `resolve_icon_path` unit test asserting the specific `OutsideRoot`
   rejection path.)

## Scenario 4 — Restart applies icon changes, no rebuild, using a private package

**Proves**: FR-007, User Story 1 scenario 3, User Story 4 scenario 2.

1. Create a private catalog package under `data/catalog/` (copy
   `catalog/public/`'s five files as a starting point — see
   `docs/data-model.md#updating-a-package` for the existing, unchanged
   package-authoring checklist).
2. Add an `assets/icons/` reference to one of its buildables, pointing
   at a PNG you place under the project's `assets/icons/` directory
   (private packages still resolve icons relative to the same shared
   `assets/icons/` root — icons are not duplicated per-package).
3. Close the app if running, launch it (no rebuild).
4. **Expected**: the icon appears immediately — no `cargo build` was run
   between adding the icon reference and seeing it.
5. Replace the PNG file's contents (same filename) with a visibly
   different image, close and relaunch.
6. **Expected**: the new image appears after the restart.

## Scenario 5 — Icons appear on all four required surfaces

**Proves**: FR-003, FR-017, User Story 3 scenarios 1-3, SC-005.

Using Scenario 1's single illustrated buildable (Xiranite Power Pole)
and at least one text-only buildable (Refinery Unit):

1. **Palette**: open the sidebar block palette. **Expected**: the
   Xiranite Power Pole row shows a small thumbnail next to its existing
   name/footprint text; the Refinery Unit row remains text-only.
2. **Single-buildable preview**: select the Xiranite Power Pole tool and
   hover the grid without clicking. **Expected**: a translucent icon
   (not the arrow, since it no longer exists) tracks the cursor inside
   the preview footprint.
3. **Blueprint preview**: save a blueprint containing both an
   illustrated and a text-only instance (see the existing "Save as
   blueprint" flow), then arm it for insertion and hover the grid.
   **Expected**: each member's own preview shows its own icon or text,
   independently, at its saved relative position and orientation.
4. **Placed instance**: place the buildable for real. **Expected**:
   same icon as the palette thumbnail, now at full opacity.

## Scenario 6 — Rotation: icon or text turns smoothly, no arrow anywhere

**Proves**: FR-009, User Story 3 scenarios 4-8, SC-006.

1. Place one illustrated instance and one text-only instance.
2. Select the illustrated instance, trigger rotation (`R` or "Rotate
   90°"). **Expected**: the icon itself visibly turns over the existing
   ~200ms transition; no arrow is drawn anywhere on or near the
   instance, before, during, or after the transition.
3. Select the text-only instance, trigger rotation. **Expected**: the
   `RU`/`CU`-style label itself visibly turns over the same transition
   window; again, no arrow.
4. Select both instances together, trigger a group rotation around
   their shared pivot. **Expected**: both representations (image and
   text) slide and turn together, synchronized, exactly as today's
   existing group-rotation position animation already does for the
   (now-removed) arrow.
5. Attempt a rotation that would go out of bounds (e.g. rotate a
   rectangular instance pinned against a base edge, if the current
   catalog's confirmed buildables allow constructing that case; a
   synthetic oversized test buildable is an acceptable substitute — see
   `tasks.md`'s test plan). **Expected**: no transition starts; the
   representation stays exactly as it was.
6. Trigger New/Open/base-change/undo/redo while a rotation transition is
   still visually in progress. **Expected**: instantaneous snap to the
   new/restored layout's resting state — no stale mid-turn image or
   text left behind, matching today's already-established Phase 8
   contract for the arrow (research.md Decision 6: `RotationVisuals`
   itself is unchanged).

## Scenario 7 — README self-sufficiency

**Proves**: FR-015, User Story 4, SC-003.

1. Starting from a clean checkout with no prior conversation context,
   follow only `README.md`'s new icon/data-customization subsection.
2. **Expected**: a reader can, using only the README, assign an icon to
   the Xiranite Power Pole, change its `display_name`, see both take
   effect after a restart, and then successfully remove the `icon`
   reference to restore text-only rendering — without needing to ask
   for help beyond the document itself.

## Out of scope for this quickstart

- Automated test names/assertions — `tasks.md`'s job.
- Exact PNG test fixture bytes/generation script — an implementation
  detail decided during task execution, not this planning artifact.
- Performance/large-catalog stress testing — spec.md's success criteria
  are correctness-shaped (SC-001 through SC-006), not throughput-shaped;
  no scenario here needs one.
