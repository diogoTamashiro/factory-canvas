use std::sync::Arc;

use super::catalog::{BuildableId, Catalog, ProductId};
use super::document::{CatalogProvenance, DocumentMetadata};
use super::geometry::{GridPoint, Rotation};
use super::layout::{BlockInstance, EntityId, FactoryLayout, PlacementError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlueprintEntityId(u64);

impl BlueprintEntityId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintId(Arc<str>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintIdError {
    MissingPrefix,
    InvalidUuid,
}

impl BlueprintId {
    pub fn parse(value: &str) -> Result<Self, BlueprintIdError> {
        let hex = value
            .strip_prefix("blueprint_")
            .ok_or(BlueprintIdError::MissingPrefix)?;
        let is_lowercase_simple_uuid = hex.len() == 32
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if !is_lowercase_simple_uuid {
            return Err(BlueprintIdError::InvalidUuid);
        }

        Ok(Self(Arc::from(value)))
    }

    pub fn generate() -> Self {
        let simple = uuid::Uuid::new_v4().simple().to_string();
        Self(Arc::from(format!("blueprint_{simple}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintNode {
    id: BlueprintEntityId,
    buildable_id: BuildableId,
    relative_origin: GridPoint,
    rotation: Rotation,
    production_target: Option<ProductId>,
}

impl BlueprintNode {
    pub const fn id(&self) -> BlueprintEntityId {
        self.id
    }

    pub fn buildable_id(&self) -> &BuildableId {
        &self.buildable_id
    }

    pub const fn relative_origin(&self) -> GridPoint {
        self.relative_origin
    }

    pub const fn rotation(&self) -> Rotation {
        self.rotation
    }

    pub fn production_target(&self) -> Option<&ProductId> {
        self.production_target.as_ref()
    }
}

/// A compass direction pointing outward from one edge of a blueprint's
/// bounding rectangle. Reuses the exact vocabulary
/// `docs/data-model.md`'s "Planned physical ports" section already
/// defines for the future catalog-level `PortDefinition` (research.md
/// Decision 3) — a distinct, simpler concept from that eventual system,
/// not a smaller reinvention of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    North,
    East,
    South,
    West,
}

/// A named boundary marker on a `Blueprint`, purely descriptive (FR-010):
/// it carries no port type, flow direction, or connection state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    name: String,
    anchor: GridPoint,
    side: Side,
}

impl Interface {
    pub fn new(name: impl Into<String>, anchor: GridPoint, side: Side) -> Self {
        Self {
            name: name.into(),
            anchor,
            side,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn anchor(&self) -> GridPoint {
        self.anchor
    }

    pub const fn side(&self) -> Side {
        self.side
    }
}

/// Why one `Interface` in a candidate list failed validation. `index`
/// identifies which element of the list failed, matching this codebase's
/// existing `node_index`/`entity_index` convention for pinpointing one
/// item in a validated collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceError {
    BlankName { index: usize },
    DuplicateName { index: usize },
    NotOnBoundary { index: usize },
}

/// The smallest axis-aligned rectangle (in the same relative coordinate
/// space as `BlueprintNode::relative_origin`) containing every node's
/// rotated footprint — the union-of-occupied-rects computation
/// `FactoryLayout`'s private `OccupiedRect::union` already implements
/// for multi-selection rotation pivots, reimplemented here over
/// `BlueprintNode`s/`BuildableDefinition`s instead of placed instances
/// (research.md Decision 3), since that layout-internal type is not
/// reachable from `blueprint.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FootprintBounds {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

impl FootprintBounds {
    fn of_nodes(nodes: &[BlueprintNode], catalog: &Catalog) -> Option<Self> {
        nodes
            .iter()
            .map(|node| {
                let footprint = catalog
                    .buildable(node.buildable_id())
                    .expect("blueprint node buildable ID was already validated at construction")
                    .footprint();
                let footprint = node.rotation().apply_to(footprint);
                let left = i64::from(node.relative_origin().x);
                let top = i64::from(node.relative_origin().y);
                Self {
                    left,
                    top,
                    right: left + i64::from(footprint.width()),
                    bottom: top + i64::from(footprint.height()),
                }
            })
            .reduce(Self::union)
    }

    fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    /// Whether `anchor`/`side` sit on this rectangle's own outer boundary:
    /// `anchor` must lie within the rectangle (inclusive of its edges),
    /// and `side` must point outward at that tile — `West` only valid on
    /// the left edge, `East` only on the right edge, `North` only on the
    /// top edge, `South` only on the bottom edge, with a corner tile
    /// allowing either of its two adjacent outward sides.
    fn contains_boundary_point(self, anchor: GridPoint, side: Side) -> bool {
        let x = i64::from(anchor.x);
        let y = i64::from(anchor.y);
        // `right`/`bottom` are exclusive (one past the last occupied
        // tile), so the last valid tile index on those edges is
        // `right - 1` / `bottom - 1`.
        let within = x >= self.left && x < self.right && y >= self.top && y < self.bottom;
        if !within {
            return false;
        }
        match side {
            Side::West => x == self.left,
            Side::East => x == self.right - 1,
            Side::North => y == self.top,
            Side::South => y == self.bottom - 1,
        }
    }
}

/// Validates a candidate interface list against `nodes`' own bounding
/// rectangle: every name must be non-blank once trimmed and unique
/// (trimmed, case-sensitive, matching this project's existing
/// `DocumentMetadata.name` comparison convention) among the list, and
/// every anchor/side must lie on that rectangle's own outer boundary
/// (research.md Decision 3). Returns the first failure found, in list
/// order — mirrors this codebase's existing "stop at the first invalid
/// item" convention (`Blueprint::from_nodes`, `decode_blueprint_document`).
fn validate_interfaces(
    interfaces: &[Interface],
    nodes: &[BlueprintNode],
    catalog: &Catalog,
) -> Result<(), InterfaceError> {
    if interfaces.is_empty() {
        return Ok(());
    }

    let bounds = FootprintBounds::of_nodes(nodes, catalog)
        .expect("nodes was already confirmed non-empty by the caller");

    for (index, interface) in interfaces.iter().enumerate() {
        if interface.name().trim().is_empty() {
            return Err(InterfaceError::BlankName { index });
        }
        let trimmed_name = interface.name().trim();
        let is_duplicate = interfaces[..index]
            .iter()
            .any(|earlier| earlier.name().trim() == trimmed_name);
        if is_duplicate {
            return Err(InterfaceError::DuplicateName { index });
        }
        if !bounds.contains_boundary_point(interface.anchor(), interface.side()) {
            return Err(InterfaceError::NotOnBoundary { index });
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintCreationError {
    EmptySelection,
    EntityNotFound { id: EntityId },
    InvalidInterface(InterfaceError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintNodeInput {
    pub id: BlueprintEntityId,
    pub buildable_id: BuildableId,
    pub relative_origin: GridPoint,
    pub rotation: Rotation,
    pub production_target: Option<ProductId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintNodeValidationError {
    EmptyNodes,
    BuildableNotFound {
        node_id: BlueprintEntityId,
        buildable_id: BuildableId,
    },
    ProductNotFound {
        node_id: BlueprintEntityId,
        product_id: ProductId,
    },
    UnsupportedProduct {
        node_id: BlueprintEntityId,
        buildable_id: BuildableId,
        product_id: ProductId,
    },
    InvalidInterface(InterfaceError),
}

/// Why a batch insertion (`Blueprint::insert_into`) was rejected as a
/// whole. `node_index` identifies which blueprint node caused the
/// failure, matching this codebase's existing `node_index`/`entity_index`
/// convention. Mirrors `PlacementError`'s variant shape one-for-one for
/// the categories a single placement can also fail with
/// (`BuildableNotFound`/`ProductNotFound`/`UnsupportedProduct`/
/// `OutOfBounds`/`Collision`), plus two variants specific to a *batch*
/// operation a single `place()` call never needs (research.md Decision 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintInsertionError {
    CoordinateOverflow {
        node_index: usize,
    },
    EntityIdsExhausted,
    BuildableNotFound {
        node_index: usize,
        buildable_id: BuildableId,
    },
    ProductNotFound {
        node_index: usize,
        product_id: ProductId,
    },
    UnsupportedProduct {
        node_index: usize,
        buildable_id: BuildableId,
        product_id: ProductId,
    },
    OutOfBounds {
        node_index: usize,
    },
    Collision {
        node_index: usize,
        conflicting_id: EntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blueprint {
    id: BlueprintId,
    provenance: CatalogProvenance,
    metadata: DocumentMetadata,
    nodes: Vec<BlueprintNode>,
    interfaces: Vec<Interface>,
}

impl Blueprint {
    pub fn from_selection(
        layout: &FactoryLayout,
        selected_ids: impl IntoIterator<Item = EntityId>,
        id: BlueprintId,
        metadata: DocumentMetadata,
        interfaces: Vec<Interface>,
    ) -> Result<Self, BlueprintCreationError> {
        let mut ids: Vec<EntityId> = selected_ids.into_iter().collect();
        ids.sort();
        ids.dedup();

        if ids.is_empty() {
            return Err(BlueprintCreationError::EmptySelection);
        }

        let mut instances = Vec::with_capacity(ids.len());
        for entity_id in ids {
            let instance = layout
                .instance(entity_id)
                .ok_or(BlueprintCreationError::EntityNotFound { id: entity_id })?;
            instances.push(instance);
        }

        let min_x = instances
            .iter()
            .map(|instance| instance.origin().x)
            .min()
            .expect("selection was already confirmed non-empty");
        let min_y = instances
            .iter()
            .map(|instance| instance.origin().y)
            .min()
            .expect("selection was already confirmed non-empty");

        let nodes: Vec<BlueprintNode> = instances
            .into_iter()
            .enumerate()
            .map(|(index, instance)| BlueprintNode {
                id: BlueprintEntityId::new(index as u64 + 1),
                buildable_id: instance.buildable_id().clone(),
                relative_origin: GridPoint::new(
                    instance.origin().x - min_x,
                    instance.origin().y - min_y,
                ),
                rotation: instance.rotation(),
                production_target: instance.production_target().cloned(),
            })
            .collect();

        validate_interfaces(&interfaces, &nodes, layout.catalog())
            .map_err(BlueprintCreationError::InvalidInterface)?;

        Ok(Self {
            id,
            provenance: CatalogProvenance::from_catalog(layout.catalog()),
            metadata,
            nodes,
            interfaces,
        })
    }

    pub fn id(&self) -> &BlueprintId {
        &self.id
    }

    pub fn provenance(&self) -> &CatalogProvenance {
        &self.provenance
    }

    pub fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    pub fn nodes(&self) -> &[BlueprintNode] {
        &self.nodes
    }

    pub fn interfaces(&self) -> &[Interface] {
        &self.interfaces
    }

    /// Every `(anchor, side)` combination lying on this blueprint's own
    /// bounding-rectangle boundary — exactly the set `validate_interfaces`
    /// accepts, computed once so a caller never has to reimplement or
    /// guess at that geometry. A corner tile contributes two entries, one
    /// per adjacent outward side. No particular ordering is guaranteed.
    ///
    /// Used by the save-as-blueprint dialog (research.md's UI scope
    /// decision) to offer a finite, always-valid set of choices instead
    /// of free-form coordinate entry or new canvas hit-testing.
    pub fn boundary_points(&self, catalog: &Catalog) -> Vec<(GridPoint, Side)> {
        let Some(bounds) = FootprintBounds::of_nodes(&self.nodes, catalog) else {
            return Vec::new();
        };
        let mut points = Vec::new();
        for x in bounds.left..bounds.right {
            for y in bounds.top..bounds.bottom {
                let anchor = GridPoint::new(x as i32, y as i32);
                for side in [Side::North, Side::East, Side::South, Side::West] {
                    if bounds.contains_boundary_point(anchor, side) {
                        points.push((anchor, side));
                    }
                }
            }
        }
        points
    }

    /// Reconstructs a `Blueprint` from already-decoded, already-canonicalized nodes
    /// (for example, from a persisted document). Every node's `buildable_id` and, if
    /// present, `production_target` are validated against `catalog` exactly like a
    /// live domain mutation would validate them; nothing is trusted from the input
    /// beyond that. `interfaces` is re-validated exactly like `from_selection`
    /// validates it (blank/duplicate name, off-boundary anchor/side) — nothing
    /// about a decoded document's `interfaces` is trusted either. Canonical
    /// local-ID numbering and node ordering are the caller's responsibility (the
    /// persistence layer enforces `1..=N` before calling this).
    pub fn from_nodes(
        id: BlueprintId,
        catalog: Catalog,
        metadata: DocumentMetadata,
        nodes: Vec<BlueprintNodeInput>,
        interfaces: Vec<Interface>,
    ) -> Result<Self, BlueprintNodeValidationError> {
        if nodes.is_empty() {
            return Err(BlueprintNodeValidationError::EmptyNodes);
        }

        let nodes: Vec<BlueprintNode> = nodes
            .into_iter()
            .map(|input| {
                let definition = catalog.buildable(&input.buildable_id).ok_or_else(|| {
                    BlueprintNodeValidationError::BuildableNotFound {
                        node_id: input.id,
                        buildable_id: input.buildable_id.clone(),
                    }
                })?;
                if let Some(product_id) = &input.production_target {
                    if catalog.product(product_id).is_none() {
                        return Err(BlueprintNodeValidationError::ProductNotFound {
                            node_id: input.id,
                            product_id: product_id.clone(),
                        });
                    }
                    if !definition.production_targets().contains(product_id) {
                        return Err(BlueprintNodeValidationError::UnsupportedProduct {
                            node_id: input.id,
                            buildable_id: input.buildable_id.clone(),
                            product_id: product_id.clone(),
                        });
                    }
                }

                Ok(BlueprintNode {
                    id: input.id,
                    buildable_id: input.buildable_id,
                    relative_origin: input.relative_origin,
                    rotation: input.rotation,
                    production_target: input.production_target,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        validate_interfaces(&interfaces, &nodes, &catalog)
            .map_err(BlueprintNodeValidationError::InvalidInterface)?;

        Ok(Self {
            id,
            provenance: CatalogProvenance::from_catalog(&catalog),
            metadata,
            nodes,
            interfaces,
        })
    }

    /// Inserts every node of `self` into `layout` as a single, atomic,
    /// all-or-nothing batch (spec FR-001 through FR-007): each node
    /// becomes a new, independent `BlockInstance` at
    /// `insertion_point + node.relative_origin`, keeping its buildable,
    /// rotation, and configured product exactly, with a fresh sequential
    /// ID starting at `first_id`. On success, `layout` now contains every
    /// new instance and the return value is the caller's new
    /// `next_entity_id` (`first_id + self.nodes().len()`). On failure,
    /// `layout` is completely unchanged — not even partially mutated
    /// (research.md Decision 2: validated on a clone, committed only on
    /// total success, the same pattern `FactoryLayout`'s private
    /// `replace_instances_atomically` already uses at a finer grain).
    pub fn insert_into(
        &self,
        layout: &mut FactoryLayout,
        insertion_point: GridPoint,
        first_id: u64,
    ) -> Result<u64, BlueprintInsertionError> {
        let node_count = self.nodes.len() as u64;
        let next_id = first_id
            .checked_add(node_count)
            .ok_or(BlueprintInsertionError::EntityIdsExhausted)?;

        let mut candidate = layout.clone();
        for (node_index, node) in self.nodes.iter().enumerate() {
            // Safe: `first_id.checked_add(node_count)` above already
            // confirmed `first_id + k` fits in `u64` for every
            // `k <= node_count`, and `node_index < node_count` here.
            let id = EntityId::new(first_id + node_index as u64);

            let x = insertion_point
                .x
                .checked_add(node.relative_origin().x)
                .ok_or(BlueprintInsertionError::CoordinateOverflow { node_index })?;
            let y = insertion_point
                .y
                .checked_add(node.relative_origin().y)
                .ok_or(BlueprintInsertionError::CoordinateOverflow { node_index })?;

            let instance = BlockInstance::new(
                id,
                node.buildable_id().clone(),
                GridPoint::new(x, y),
                node.rotation(),
            )
            .with_production_target(node.production_target().cloned());

            candidate.place(instance).map_err(|error| match error {
                PlacementError::DuplicateEntityId { .. } => unreachable!(
                    "first_id must come from the destination's next_entity_id allocator, \
                     already an upper bound on every existing ID; a freshly allocated \
                     sequential ID colliding is a caller contract violation, not a \
                     legitimate insertion failure"
                ),
                PlacementError::BuildableNotFound { buildable_id, .. } => {
                    BlueprintInsertionError::BuildableNotFound {
                        node_index,
                        buildable_id,
                    }
                }
                PlacementError::ProductNotFound { product_id, .. } => {
                    BlueprintInsertionError::ProductNotFound {
                        node_index,
                        product_id,
                    }
                }
                PlacementError::UnsupportedProduct {
                    buildable_id,
                    product_id,
                    ..
                } => BlueprintInsertionError::UnsupportedProduct {
                    node_index,
                    buildable_id,
                    product_id,
                },
                PlacementError::OutOfBounds { .. } => {
                    BlueprintInsertionError::OutOfBounds { node_index }
                }
                PlacementError::Collision { conflicting_id, .. } => {
                    BlueprintInsertionError::Collision {
                        node_index,
                        conflicting_id,
                    }
                }
            })?;
        }

        *layout = candidate;
        Ok(next_id)
    }
}
