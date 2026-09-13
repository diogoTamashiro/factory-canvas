# Feature Specification: Custom Buildable Icons

**Feature Branch**: `008-custom-buildable-icons`

**Created**: 2026-09-13

**Status**: Draft — clarified and ready for planning

**Input**: User description: "implementar adição de imagem para as máquinas/itens, como o poste de xiranita, deixar o texto como default se não tiver imagem, ter um repo dentro do projeto só para essas imagens, ele deve ser aplicado por meio do json objeto das máquinas que já existe no projeto, documentar em readme que os icones das máquinas e os dados delas são customizáveis pelo usuário"

## Clarifications

### Session 2026-09-13

- Q: Where should icons appear? → A: On placed canvas instances, placement previews (including blueprints), and the buildable palette. Textual control names remain available. (FR-003, FR-011, FR-017)
- Q: What does an image repository inside the project mean? → A: One dedicated versioned directory inside the existing project; no separate Git repository or submodule. (FR-008)
- Q: How should icons, text, and the placeholder arrow behave on rotation? → A: Remove the arrow entirely. With an image, rotate the image smoothly; without one, rotate the fallback text smoothly instead. This interpretation was explicitly confirmed. (FR-009)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Recognize a placed buildable by a custom icon (Priority: P1)

A player supplies an image for a buildable, such as the Xiranite Power Pole, and associates it with that buildable's existing catalog definition. After restarting the application, every placed instance of that type shows the chosen icon instead of its abbreviated canvas text. The player can replace the icon without editing application code or recreating the factory.

**Why this priority**: This is the requested visual customization and provides value even when only one buildable has an image.

**Independent Test**: Associate a user-supplied image with the existing Xiranite Power Pole definition, leave another buildable without an image, restart, and place both. Check the customized icon, the other buildable's textual representation, and unchanged placement behavior.

**Acceptance Scenarios**:

1. **Given** an otherwise-valid catalog containing a buildable with a usable local static PNG image reference, **When** the player starts the application and places that buildable, **Then** its image replaces the abbreviated text inside its canvas footprint.
2. **Given** multiple instances of the same buildable, including instances reopened from a factory or inserted from a blueprint, **When** those instances appear, **Then** they all use the icon associated with that buildable in the active catalog.
3. **Given** a user-maintained catalog and its image collection, **When** the player replaces an icon while the application is closed and restarts it, **Then** the new icon appears without rebuilding the application, modifying application code, or resaving existing documents.
4. **Given** an image whose proportions differ from the buildable's footprint, **When** the image is displayed at rest or during a turn, **Then** it fits inside the displayed footprint without stretching or cropping, preserves transparency, and does not hide the footprint boundary or selection indication.
5. **Given** a complete project containing its dedicated versioned image directory and a user-maintained catalog with relative image references, **When** the player moves that complete collection to another local location and launches it using the documented instructions, **Then** the same images resolve without editing those references or setting up a second repository.

---

### User Story 2 - Keep editing when an icon is absent or unusable (Priority: P1)

A player can use existing text-only catalogs and partially illustrated catalogs. Forgetting an image file, moving the project, or supplying a broken image must not make a machine disappear or prevent editing an otherwise-valid factory.

**Why this priority**: Text fallback is an explicit requirement and keeps optional presentation assets from becoming a new dependency for existing user data.

**Independent Test**: Use an otherwise-valid catalog with one working icon and several absent or unusable icon references. Confirm that only the affected representations fall back and all buildables remain available.

**Acceptance Scenarios**:

1. **Given** an existing valid catalog with no image references, **When** the application starts, **Then** every placed instance retains its configured abbreviated text without requiring catalog edits; the former arrow is absent, and the text itself now expresses the instance's orientation.
2. **Given** an omitted, null, or blank image reference, or a referenced file that is missing, unreadable, corrupt, unsupported, or outside the permitted image collection, **When** the buildable is displayed, **Then** it uses its textual fallback and remains selectable, placeable, movable, and removable.
3. **Given** one unusable image and another usable image in an otherwise-valid catalog, **When** the application starts, **Then** the valid icon still appears, the catalog's data remains active, and the failed asset does not trigger replacement of the whole catalog.
4. **Given** an explicit image reference that cannot be used, **When** the player checks application feedback, **Then** a non-blocking sanitized warning is available without exposing absolute paths or private catalog values; simply omitting an image produces no warning.
5. **Given** malformed catalog JSON, a non-textual image reference other than null, or invalid required machine data, **When** the application starts, **Then** the existing all-or-nothing catalog validation and safe startup fallback still apply; optional icons do not relax those rules.
6. **Given** an image reference targeting an absolute path, a network location, a parent directory, or a linked file outside the image collection, **When** the application resolves that reference, **Then** it reads none of that external resource, uses text for the affected buildable, and keeps otherwise-valid machine data active.
7. **Given** an otherwise-valid saved factory and a saved blueprint, **When** their optional image files are absent, **Then** the factory opens and the blueprint remains insertable with textual fallback; neither saved document is rewritten simply because the artwork is unavailable.

---

### User Story 3 - Keep recognition and orientation consistent while editing (Priority: P2)

A player sees the same machine icon on placed instances, placement previews (including each blueprint member), and the buildable palette. The former placeholder arrow is removed entirely. On the canvas, the icon itself turns smoothly; if an image is unavailable, the abbreviated text itself turns smoothly instead. Names in the palette and companion instance list remain readable and accessible.

**Why this priority**: The chosen appearance must stay consistent throughout the CAD workflow, and optional images must not leave text-only machines without the requested visual rotation.

**Independent Test**: Use a mixed icon/text layout and a blueprint containing both types. Check the palette, each kind of placement preview, accepted and rejected single/group rotations, rapid turns, and layout restoration. Confirm that no arrow remains and that the correct representation is animated without changing edit outcomes.

**Acceptance Scenarios**:

1. **Given** a buildable with a usable icon, **When** the player finds it in the palette, **Then** an unrotated thumbnail accompanies its existing name and footprint label, and the control retains its selection behavior and accessible name. A buildable without an icon retains a text-only palette control.
2. **Given** a buildable armed for placement, **When** the player hovers over the grid, **Then** its normal-placement orientation is shown using a translucent icon or textual fallback inside the preview footprint, without an arrow and without prevalidating the candidate.
3. **Given** an armed blueprint containing multiple buildables and orientations, **When** the player hovers over the grid, **Then** every member's preview uses its own associated icon or textual fallback at its saved relative position and orientation. One missing image does not suppress another member's icon or the blueprint preview.
4. **Given** a placed illustrated instance or a text-only instance, **When** an accepted single-instance rotation occurs, **Then** the icon or the text itself turns smoothly to the accepted orientation without an arrow; the settled depiction agrees with the stored rotation.
5. **Given** a selected group containing both illustrated and text-only instances, **When** an orbital rotation succeeds, **Then** every member's chosen representation turns and changes position together during the existing transition window, finishing at the accepted position and orientation.
6. **Given** a rejected edit, **When** the player attempts placement, movement, or rotation, **Then** the image does not make the action appear accepted. A rejected rotation starts no new transition; an already-running accepted transition is not restarted or canceled by that rejection.
7. **Given** a turn in progress, **When** the player requests another accepted turn, **Then** icons and rotated text continue from their current visual states rather than snapping to a stale angle or waiting in a queue.
8. **Given** a turn in progress, **When** New/Open, a base change, undo, or redo replaces the layout, **Then** the old transition is discarded and the resulting layout appears immediately at its correct positions and orientations, with no arrow or stale depiction. An ordinary move also remains instantaneous.
9. **Given** an image-bearing instance, **When** the player selects it or uses the companion instance list through keyboard or assistive technology, **Then** its name, selection state, and existing instance details remain available just as for a text-only instance.

---

### User Story 4 - Customize icons and machine data using the README (Priority: P2)

A player can discover that both icons and machine data are customizable and follow the project README to make a local change safely, without asking an assistant to explain hidden setup steps.

**Why this priority**: Customization is not usable if its storage location, catalog association, restart requirements, or fallback behavior are only documented in conversation history.

**Independent Test**: Starting with the documented public sample, follow the README to prepare a user-maintained catalog, associate a supplied icon with an existing buildable, change its display name, restart, and then remove the icon reference to restore text.

**Acceptance Scenarios**:

1. **Given** a reader unfamiliar with customization, **When** they read the README, **Then** it explicitly states that machine/item icons and catalog-defined machine data are user-customizable and explains where to place images and how the existing machine JSON object refers to them.
2. **Given** the documented example and a user-supplied image, **When** the reader follows the setup and restart instructions, **Then** the existing Xiranite Power Pole can show that icon and a customized display name without a code change.
3. **Given** a reader who wants to change machine data, **When** they follow the guide, **Then** the guide identifies the supported editable fields, preserves stable identifiers, explains validation and data-version responsibilities, and distinguishes user-maintained data from the bundled public fallback.
4. **Given** a missing or rejected icon, **When** the reader follows troubleshooting instructions, **Then** they can identify the supported format and location rules or remove the reference to return to the text-only default.

### Edge Cases

- A catalog contains no icons, only some icons, or several buildables intentionally sharing one image: each buildable independently resolves to its configured image or text.
- An icon-only edit changes no factory content: it creates no document edit or undo-history entry and requires no document resave.
- Moving the complete project/catalog/image collection together must preserve valid relative associations; losing the image collection must still leave valid machine data usable with text.
- A path attempts to access an absolute location, a parent directory, a network resource, or an indirectly linked file outside the allowed collection: it must never read that external resource and must use the local fallback.
- Omitted, null, or blank image references mean no image and produce no warning. Any other non-textual reference value is malformed catalog data and remains subject to strict catalog validation.
- Image transparency or proportions do not redefine a machine's footprint or clickable area. During rotation, the icon or text remains contained within the displayed footprint and never obscures its boundary or selection indication. Invalid image content must never be interpreted as executable content.
- Changing image files while the application is running does not promise a live update; users close the application before editing and restart afterward, as with the existing catalog workflow.
- New/Open, a base change, undo, and redo during a visual rotation retain their existing immediate-reset behavior; no pending icon transition survives a whole-layout replacement.
- Rapid repeated rotations start from the current visual state rather than queuing stale turns. Panning and zooming keep icons and rotated text attached to their instances. A plain move stays instantaneous; it does not acquire the position animation of a group rotation.
- An icon or textual symbol may be visually symmetric. Its applied angle must still follow the stored orientation; this feature does not add a replacement arrow or require artwork whose direction is recognizable.
- An unrotated single-buildable preview and palette thumbnail use the image's original orientation. A blueprint preview uses each member's saved relative position and orientation; individual missing images fall back independently.
- A preview may extend beyond the base or overlap an instance: adding images or text does not prevalidate it or promise that placement will succeed. Existing preview suppression during destructive confirmations remains unchanged.
- A factory or blueprint opened without its author's custom images remains usable when its required catalog data is valid. Documents do not carry or require the author's local image files.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to associate an optional local image with each buildable through that buildable's existing JSON catalog object, without a second per-machine mapping or a code change. The association belongs to the buildable type, not an individual placed instance.
- **FR-002**: A placed instance with a usable image MUST display that image instead of its abbreviated canvas text. A placed instance without a usable image MUST retain its configured abbreviated text as the default representation.
- **FR-003**: Icons MUST appear on placed canvas instances, single-buildable placement previews, each member of a blueprint placement preview, and buildable-palette entries. The companion instance list MUST retain its existing textual representation; adding thumbnails to that list, product choices, or blueprint-library rows is outside this increment.
- **FR-004**: Omitted, null, blank, missing-file, unreadable, corrupt, unsupported, and disallowed optional image references MUST resolve independently to text rather than prevent editing, remove a buildable, or replace otherwise-valid catalog data. Intentionally unconfigured images MUST be silent; unusable explicit references MUST provide non-blocking sanitized feedback.
- **FR-005**: Image access MUST stay offline and within the designated local image collection, including when a reference indirectly points elsewhere. No image reference may cause network access, arbitrary filesystem reads, or execution of image content.
- **FR-006**: Images MUST preserve their proportions and transparency, fit entirely inside the displayed physical footprint without cropping or stretching, and keep footprint boundaries and selection indications visible. Both icons and fallback text MUST remain contained and keep those markings visible during rotation. Their pixels MUST NOT determine hit testing or occupied space.
- **FR-007**: Users MUST be able to add, replace, remove, or reassociate images and edit supported machine data in their user-maintained package while the app is closed, then apply the changes by restarting without rebuilding the application. This feature MUST NOT add live reload or automatic downloads.
- **FR-008**: The project MUST provide one dedicated, documented, versioned image directory inside the existing project, separate from machine definitions and saved factories. It MUST NOT require a separate image repository, submodule, additional checkout, or network setup. Image associations MUST be relative to this collection so moving the complete project preserves them.
- **FR-009**: The placeholder orientation arrow MUST be removed completely. A placed instance MUST orient its icon, or its fallback text when no icon is usable, according to its stored rotation and animate that same representation smoothly on an accepted rotation. A group rotation MUST animate every selected member's position and orientation together, including mixed icon/text groups. Rejected rotations MUST start no new transition; rapid repeats MUST continue from the current visual state. Plain moves and whole-layout restoration (New/Open, base change, undo/redo) MUST remain instantaneous. Palette thumbnails MUST remain in their original, unrotated orientation.
- **FR-010**: Adding or changing an image MUST NOT change machine identity, physical footprint, bounds/collision validation, selection behavior, production configuration, or document/history contents. Existing valid edit outcomes MUST remain the same for illustrated and text-only buildables.
- **FR-011**: Palette icons MUST supplement the existing textual name and footprint label, not replace them. A palette entry without an icon MUST remain a text-only control. Icons MUST NOT remove selection semantics or keyboard/assistive-technology access from the palette or the unchanged textual instance list.
- **FR-012**: Existing valid catalogs with no image association MUST continue to load unchanged. The optional association may be omitted, null, or blank to select text. Other non-textual values, malformed JSON, unknown unrelated fields, and invalid required machine data MUST retain the established strict validation and safe startup behavior.
- **FR-013**: Existing factories and blueprints MUST use the active catalog's current image associations without requiring image metadata, embedded artwork, or local file locations in saved documents. Missing optional images MUST NOT prevent loading otherwise-valid documents.
- **FR-014**: This increment MUST support static PNG icons, including transparency. Other image formats MUST use the same safe fallback as other unsupported assets. The documentation MUST state the supported format rather than imply support for arbitrary images.
- **FR-015**: The README MUST explicitly document customization of both icons and machine data, the selected image-storage arrangement, the optional association in the existing machine JSON object, a complete example using an existing confirmed buildable, supported image format, stable identifiers, data-version/validation responsibilities, restart requirements, restoring text, and troubleshooting. It MUST distinguish changes to user-maintained assets/data from changes to the bundled public catalog that still require a rebuild.
- **FR-016**: The feature MUST remain usable without shipping official game artwork or private reference data. Public examples MUST use only already-confirmed public machine data and, if any images are bundled, redistributable illustrative assets. User-supplied private images MUST remain local; acquiring a complete icon collection is not an acceptance prerequisite.
- **FR-017**: Single-buildable and blueprint-member previews MUST show a translucent icon or textual fallback inside each preview footprint, without an arrow, at the candidate's own position and orientation. Preview opacity, footprint boundaries, cursor tracking, and suppression during destructive confirmations MUST retain their existing intent; icons and labels MUST NOT predict or prevalidate placement acceptance. Preview content MUST follow the hovered candidate directly, not start a placement animation.

### Key Entities

- **Buildable definition**: The existing machine/item description, with stable identity, editable display information, physical footprint and capabilities, plus an optional image association.
- **Icon asset**: A user-supplied static image that illustrates a buildable without defining its physical or production behavior.
- **Image collection**: One dedicated directory versioned with the existing project. It is the documented root for relative icon associations and does not require a separate repository.
- **Textual fallback**: The buildable's configured abbreviated label, now oriented and animated directly on the canvas instead of accompanied by an arrow; palette controls retain their ordinary text-only form when no image is available.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a mixed catalog, 100% of placed instances with usable configured icons show the correct icon, and 100% of instances without usable icons remain identifiable through their configured text; no buildable becomes invisible because its image is missing.
- **SC-002**: All documented icon-failure cases leave otherwise-valid machine data and documents usable, with zero application terminations, zero external-resource accesses, and zero loss of factory content.
- **SC-003**: A reader following only the README can complete three actions with an existing buildable: assign an icon, change its display name, and restore text-only rendering. Each takes effect after a restart with no application-code changes or support instructions outside the guide.
- **SC-004**: Existing text-only catalogs and documents remain usable without manual conversion; illustrated and text-only versions of the same layout have identical placement, movement, rotation, selection, and save/reopen outcomes in all acceptance scenarios.
- **SC-005**: Across all four required surfaces (placed instances, single-buildable previews, blueprint-member previews, and palette entries), every configured image association resolves consistently. Every previously named selectable control remains identifiable and operable by keyboard and assistive technology.
- **SC-006**: In single and group rotations of mixed icon/text layouts, every canvas icon or fallback label ends at the accepted position and orientation, with no arrows present. Rejected rotations start no new transition; plain moves and whole-layout restoration leave no stale animated representation.

## Assumptions

- "Machines/items" means buildables that can already be placed on the canvas, including power poles. This increment does not introduce product/material icons, new constructible categories, or unconfirmed game entities.
- The player supplies their own artwork. The feature provides the association, storage convention, display behavior, and instructions; sourcing, extracting, or downloading official icons is out of scope.
- Static PNG is the initial supported format because small icons commonly need transparency. Animated images, vector-image support, 3D assets, and a general asset editor are outside this increment.
- Customization uses the existing closed-app catalog-edit/restart workflow and one dedicated image directory versioned with the project, not a second repository. It does not add an in-app catalog editor, drag-and-drop image picker, asset marketplace, or network service.
- Physical dimensions and capabilities remain user-maintained catalog data subject to existing validation. The README explains how to customize them without inventing their correct in-game values.
- Image-only presentation changes do not modify saved factories or blueprints. Changing machine data itself can still affect validation of a document, exactly as in the current catalog model.
- Existing CAD navigation, atomic group edits, blueprint reuse, undo/redo, semantic controls, and rotation timing are dependencies to preserve. The intentional presentation changes are icons on the four selected surfaces, text fallback on previews, and animating the icon or text directly in place of the removed arrow.
- Completing the earlier test-file modularization and repairing unrelated stale documentation are separate work, not part of this feature.
