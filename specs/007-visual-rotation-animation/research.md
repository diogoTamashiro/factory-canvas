# Research: Visual Rotation Animation

No item in Technical Context was marked `NEEDS CLARIFICATION` — every
ambiguity was resolved through four clarifying questions before spec.md
was written. This document instead records the technical decisions made
during planning, each backed by a disposable spike or direct source
inspection (`docs/engineering-standards.md` §YAGNI: "treat spikes as
disposable" — the spike file used for Decision 1 was written, run, and
deleted before this document was finalized, following the exact same
pattern Phase 7's `research.md` already established).

## Decision 1: Use `egui::Context::animate_value_with_time`, not a custom timer

**Decision**: Drive every animated value (an accumulated rotation angle
in degrees, and each axis of an animated origin) through
`egui::Context::animate_value_with_time(id, target, duration)` — a
built-in API this project's existing `egui` 0.36.1 dependency already
ships. No new crate, no `std::time::Instant`/`SystemTime` bookkeeping,
no custom easing function.

**Rationale**: A disposable spike (`tests/spike_rotation_animation.rs`,
two tests, deleted after answering the open questions below) confirmed,
against the real `egui` 0.36.1 dependency this project already has,
every behavior this feature needs:

- **First call returns the target immediately** (no animation lag for
  an instance's very first frame): calling
  `animate_value_with_time(id, 0.0, 0.2)` on a fresh `Id` at `t=0.0`
  returned `0.0` directly, not a ramp from some default.
- **Linear interpolation between calls**: retargeting to `100.0` at
  `t=0.0` and reading again at `t=0.1` (halfway through a `0.2`s
  duration) returned `54.17` — matching linear interpolation from the
  value in flight toward the new target (the small offset above the
  exact midpoint of 50.0 is `egui`'s own half-frame forward
  extrapolation, documented directly in its source).
- **Mid-flight retargeting continues from the CURRENT position, never
  restarts or jumps** — this is the exact mechanism spec FR-006
  requires. Retargeting from `100.0` to `40.0` while a transition was
  already 50% of the way toward `100.0` produced smooth continued
  motion toward the new target from wherever the value actually was,
  confirmed by inspecting `epaint`'s (a transitive `egui` crate)
  `ValueAnim`: on every call where the requested target differs from
  the previously requested target, it sets
  `anim.from_value = current_value` (the animation's live interpolated
  position at that instant) before changing `anim.to_value`. No extra
  code is needed to satisfy FR-006 — it is `animate_value_with_time`'s
  native behavior.
- **`animation_time == 0.0` forces an instant snap**: confirmed directly
  in the same source (`if animation_time == 0.0 { anim.from_value =
  value; anim.to_value = value; }`). This is the mechanism Decision 3
  below uses to make undo/redo, New, Open, and a base change perfectly
  instant (FR-008, FR-009) without disabling or resetting `egui`'s
  animation system globally.

**Alternatives considered**:

- **A custom animation struct storing `start_value`/`end_value`/
  `start_time: std::time::Instant` per entity**: strictly more code for
  identical behavior to what `egui` already provides natively, and it
  would need to reinvent exactly the retargeting-from-current-position
  logic the spike already confirmed `egui` does correctly. Rejected —
  this project's engineering standards favor minimal dependencies and
  minimal new code (`docs/engineering-standards.md` §KISS/YAGNI); reuse
  the framework's own animation primitive rather than parallel it.
- **A third-party animation/tweening crate**: no such dependency exists
  in this project today, and `egui`'s own facility is already
  sufficient — adding one would violate Constitution Principle I
  (explicit dependency policy) without justification.

## Decision 2: A per-entity monotonically-accumulating target angle, never a "shortest path" calculation

**Decision**: Track each instance's animated rotation as an
ever-increasing target in degrees (`0.0`, `90.0`, `180.0`, `270.0`,
`360.0`, `450.0`, ...), bumped by exactly `+90.0` every time that
instance's stored `Rotation` is detected to differ from its
previous value, rather than computing a "shortest angular path"
between two `Rotation` values (which would require deciding whether
`Zero -> Clockwise270` should visually spin forward 270° or backward
90°).

**Rationale**: Every rotation this application's domain ever performs —
both `FactoryLayout::rotate_instance` (single instance) and
`FactoryLayout::rotate_instances_clockwise_about` (orbital group
rotation) — always advances exactly one `Rotation::clockwise()` step
per call, confirmed directly in `src/domain/layout.rs`: the former is
invoked by `egui_app.rs` with `current.rotation().clockwise()`
explicitly, and the latter's own transform closure calls
`instance.rotation().clockwise()` unconditionally for every member it
touches. There is no code path in this application, today or reasonably
anticipated, that ever rotates counter-clockwise or by more than one
step at a time. Given that guarantee, "always animate exactly +90° in
the forward (clockwise) direction whenever a rotation is detected" is
not just simpler than shortest-path logic — it is the only choice that
can ever be visually correct here, since a real rotation command in
this app is definitionally always a single forward quarter-turn.

Accumulating without ever normalizing modulo 360 avoids reintroducing
the exact ambiguity this decision sidesteps (a modulo'd value crossing
`0`/`360` would need the same "which direction is shorter" decision this
project has no use for). The accumulated value is only reduced modulo
360 at the point of actually painting the arrow's angle — never in the
stored or animated value itself.

**Bound check**: `f32` retains exact integer precision up to `2^24`
(~16.7 million). At even a sustained 10 rotations per second on one
instance — far beyond realistic interactive use — reaching that bound
would take roughly 465 hours of continuous, uninterrupted rotation.
Accepted as a non-issue rather than engineered around.

**Alternatives considered**:

- **Compute the shortest signed angular delta between two `Rotation`
  values and animate that**: adds real complexity (a sign decision, a
  wraparound case at the 180°/-180° boundary) to correctly handle a
  case — counter-clockwise or multi-step rotation — that provably never
  happens in this codebase today. Rejected as unneeded complexity
  (YAGNI) for a scenario with no current or anticipated caller.

## Decision 3: An explicit instant-resync signal, not layout-diffing alone, distinguishes an animated rotation from undo/redo/New/Open/base-change

**Decision**: `egui_canvas.rs`'s new `RotationVisuals` type exposes a
`resync(&mut self, layout: &FactoryLayout)` method that `egui_app.rs`
calls immediately after mutating `self.layout` in exactly the four
places that replace it wholesale outside of a normal accepted rotation
— `new_document_at`, `open_document_from`, `replace_base`, and
undo/redo's shared `apply_restored_snapshot`. `resync` rebuilds
`RotationVisuals`'s internal per-entity bookkeeping to exactly match the
new layout and arms a one-shot "next frame must snap instantly" flag; the
very next `egui_canvas::show()` call consumes that flag, calling
`animate_value_with_time(..., 0.0)` (Decision 1's confirmed instant-snap
path) for every instance instead of the normal animation duration, then
clears the flag for all subsequent frames.

**Rationale**: `egui_canvas::show()` receives only `&FactoryLayout` each
frame — a pure snapshot of current state with no record of *which*
command produced it. Diffing the previous frame's stored rotation
against the current one cannot, by itself, distinguish "a rotation
command was just accepted" (should animate, per FR-003/FR-004) from "the
whole layout was just replaced by undo, redo, New, Open, or a base
change" (must NOT animate, per FR-008/FR-009) — from the canvas's point
of view both look identical: a `Rotation` value changed since last
frame. An explicit signal from the one place that actually knows which
kind of change occurred (`egui_app.rs`, which already owns every one of
these six call sites) is the only reliable way to make that distinction,
and it costs nothing extra at the three FR-008 call sites since they
already fully replace `self.layout` in one step; undo/redo needs the
identical treatment even though spec.md's Edge Cases only phrase FR-009
as "no transition animation" rather than spelling out the mechanism —
without an explicit resync, restoring an older stored rotation via undo
would look exactly like a real rotation to this feature's own
change-detection and incorrectly animate backward, which is precisely
the outcome FR-009 rules out.

**Alternatives considered**:

- **Have the domain expose *why* a layout changed (a "last operation"
  tag)**: would require a new field or event type on `FactoryLayout` or
  `FactoryCanvasApp` purely to serve this one presentation feature,
  contradicting FR-010 ("this feature MUST NOT change any domain...
  behavior") and this project's domain/UI separation principle
  (`docs/architecture.md`). Rejected.
- **Globally disable/reset `egui`'s entire `AnimationManager` via
  `Context::clear_animations()` at the four call sites**: this is a
  real, simpler-looking alternative that was seriously considered.
  Rejected because it is global — it would also discard any *unrelated*
  in-flight `egui` animation the wider UI might have going (for example
  a future hover/press animation on an unrelated widget), which this
  feature has no business touching. A per-entity, per-feature resync
  confined to `RotationVisuals`'s own bookkeeping is more precise and
  does not risk an unintended side effect on other widgets sharing the
  same `Context`.

## Decision 4: Detect "this instance was just rotated" per-entity via its own `Rotation` value, and gate position-animation on that same signal

**Decision**: For each instance, on every `show()` call, compare its
current domain `Rotation` against the value `RotationVisuals` has
stored for it. If — and only if — that instance's own `Rotation` value
has changed (and no instant-resync flag is active, per Decision 3),
treat this instance as "just rotated": bump its accumulated target angle
by `+90.0` (Decision 2) **and** animate its origin from the old stored
value to the new one over the same transition window, using the same
mechanism as the angle. If an instance's origin changes on some frame
**without** its own `Rotation` value having changed, render it at its
current origin directly, with no interpolation — exactly like every
instance renders today.

**Rationale**: Spec FR-004 requires animating position **only** for a
rotation-driven move (the orbital group case), never for a plain
arrow-key/drag move — nothing in this spec asks for animating ordinary
movement, and doing so anyway would be unrequested scope creep with no
test coverage backing it. The two domain operations that ever change an
instance's origin are `move_instances_by` (plain move — passes
`instance.rotation()` through completely unchanged, confirmed directly
in `src/domain/layout.rs`) and `rotate_instances_clockwise_about`
(orbital rotation — advances `.rotation().clockwise()` for every member
it touches, by construction, every single time). This makes "did this
specific instance's own `Rotation` enum value change" a fully reliable,
domain-guaranteed per-entity signal for "was this instance just
rotated" — there is no path in this codebase where a plain move changes
`Rotation`, and no path where either rotation operation leaves any
touched member's `Rotation` unchanged. No new domain field, method, or
event is needed to make this distinction (FR-010).

**Alternatives considered**:

- **Animate origin whenever it changes, regardless of cause**: would
  make ordinary arrow-key movement start sliding smoothly too — a
  visible behavior change to a part of the editor entirely outside this
  feature's scope (spec.md only discusses rotation), and specifically
  not what the fifth clarifying question asked for. Rejected.
- **A separate explicit "this call is a rotation batch" parameter
  threaded from `egui_app.rs` into `egui_canvas::show()`**: would work,
  but adds a new parameter to an already-parameter-heavy function for
  information the per-entity `Rotation`-change signal already provides
  for free, with no additional plumbing. Rejected as unnecessary
  (YAGNI) once the per-entity signal was confirmed reliable.

## Decision 5: Stale bookkeeping entries are pruned opportunistically, not tracked with a separate lifecycle

**Decision**: `RotationVisuals`'s per-entity map is rebuilt from scratch
on every `resync()` call (Decision 3), which naturally discards entries
for any instance no longer present. On ordinary frames, entries for
`EntityId`s no longer present in the current layout are dropped as part
of the same per-instance pass `show()` already performs — no separate
cleanup pass, timer, or capacity limit.

**Rationale**: `EntityId` is never reused after removal (a standing
project invariant), so a stale entry can never collide with, or be
mistaken for, a later different instance — the only downside of not
pruning would be unbounded memory growth over a very long single
session with many place/remove cycles. Given this project's documented
scale (tens to low hundreds of entities) and that `resync()` already
provides a full, correctness-motivated rebuild point at four common
interaction boundaries (New, Open, base change, any undo/redo), an
additional prune during the already-existing per-instance iteration in
`show()` fully addresses the growth concern with no new state, timer, or
configuration.

**Alternatives considered**:

- **A capped LRU cache**: unneeded complexity for a bound this
  project's own documented scale does not require, and it would risk
  incorrectly evicting a still-live, simply infrequently-rotated
  instance's bookkeeping. Rejected (YAGNI).

## Mechanical check: FR-005 (a rejected rotation must never animate)

Both `rotate_selected_clockwise`'s branches already only call
`self.history.record(...)` and update `self.notice`/`self.layout` on the
`Ok` arms; the `Err(error)` arm only sets
`self.notice = EditorNotice::InstanceEditRejected(error)` and leaves
`self.layout` completely untouched (confirmed by reading the existing
function body). Since `RotationVisuals` only ever detects a change by
comparing against the actual `FactoryLayout` passed into `show()`, a
rejected rotation — which never mutates that layout at all — produces
no detectable change and therefore triggers no animation, automatically,
with no extra code required to special-case it.
