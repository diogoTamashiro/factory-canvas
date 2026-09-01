use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::catalog::{
    BaseDefinition, BaseId, BuildableDefinition, BuildableId, Catalog, ProductId,
};
use super::geometry::{GridPoint, GridSize, Rotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(u64);

impl EntityId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInstance {
    id: EntityId,
    buildable_id: BuildableId,
    production_target: Option<ProductId>,
    origin: GridPoint,
    rotation: Rotation,
}

impl BlockInstance {
    pub fn new(
        id: EntityId,
        buildable_id: BuildableId,
        origin: GridPoint,
        rotation: Rotation,
    ) -> Self {
        Self {
            id,
            buildable_id,
            production_target: None,
            origin,
            rotation,
        }
    }

    pub const fn id(&self) -> EntityId {
        self.id
    }

    pub fn buildable_id(&self) -> &BuildableId {
        &self.buildable_id
    }

    pub fn production_target(&self) -> Option<&ProductId> {
        self.production_target.as_ref()
    }

    pub const fn origin(&self) -> GridPoint {
        self.origin
    }

    pub const fn rotation(&self) -> Rotation {
        self.rotation
    }

    fn transformed(&self, origin: GridPoint, rotation: Rotation) -> Self {
        Self {
            id: self.id,
            buildable_id: self.buildable_id.clone(),
            production_target: self.production_target.clone(),
            origin,
            rotation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionTargetError {
    EntityNotFound {
        id: EntityId,
    },
    ProductNotFound {
        product_id: ProductId,
    },
    UnsupportedProduct {
        buildable_id: BuildableId,
        product_id: ProductId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProductionTargetValidationError {
    ProductNotFound {
        product_id: ProductId,
    },
    UnsupportedProduct {
        buildable_id: BuildableId,
        product_id: ProductId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedInstance<'a> {
    instance: &'a BlockInstance,
    definition: &'a BuildableDefinition,
    effective_footprint: GridSize,
}

impl<'a> ResolvedInstance<'a> {
    pub const fn instance(self) -> &'a BlockInstance {
        self.instance
    }

    pub const fn definition(self) -> &'a BuildableDefinition {
        self.definition
    }

    pub const fn effective_footprint(self) -> GridSize {
        self.effective_footprint
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlacementError {
    DuplicateEntityId {
        id: EntityId,
    },
    BuildableNotFound {
        id: EntityId,
        buildable_id: BuildableId,
    },
    ProductNotFound {
        id: EntityId,
        product_id: ProductId,
    },
    UnsupportedProduct {
        id: EntityId,
        buildable_id: BuildableId,
        product_id: ProductId,
    },
    OutOfBounds {
        id: EntityId,
    },
    Collision {
        id: EntityId,
        conflicting_id: EntityId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceEditError {
    EntityNotFound {
        id: EntityId,
    },
    OutOfBounds {
        id: EntityId,
    },
    Collision {
        id: EntityId,
        conflicting_id: EntityId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpatialValidationError {
    OutOfBounds,
    Collision { conflicting_id: EntityId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OccupiedRect {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

impl OccupiedRect {
    fn from_instance(instance: &BlockInstance, footprint: GridSize) -> Self {
        let origin = instance.origin();
        let footprint = instance.rotation().apply_to(footprint);
        let left = i64::from(origin.x);
        let top = i64::from(origin.y);

        Self {
            left,
            top,
            right: left + i64::from(footprint.width()),
            bottom: top + i64::from(footprint.height()),
        }
    }

    fn is_within(self, bounds: GridSize) -> bool {
        self.left >= 0
            && self.top >= 0
            && self.right <= i64::from(bounds.width())
            && self.bottom <= i64::from(bounds.height())
    }

    fn overlaps(self, other: Self) -> bool {
        self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom
    }

    fn contains(self, point: GridPoint) -> bool {
        let x = i64::from(point.x);
        let y = i64::from(point.y);

        self.left <= x && x < self.right && self.top <= y && y < self.bottom
    }

    fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    fn center_toward_top_left(self) -> GridPoint {
        let x = (self.left + self.right) / 2;
        let y = (self.top + self.bottom) / 2;

        GridPoint::new(
            i32::try_from(x).expect("validated layout bounds fit in i32"),
            i32::try_from(y).expect("validated layout bounds fit in i32"),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactoryLayout {
    catalog: Catalog,
    base_id: BaseId,
    instances: BTreeMap<EntityId, BlockInstance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutCreationError {
    BaseNotFound { base_id: BaseId },
}

impl fmt::Display for LayoutCreationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BaseNotFound { base_id } => write!(
                formatter,
                "base '{}' does not exist in the catalog",
                base_id.as_str()
            ),
        }
    }
}

impl std::error::Error for LayoutCreationError {}

impl FactoryLayout {
    pub fn new(catalog: Catalog, base_id: BaseId) -> Result<Self, LayoutCreationError> {
        if catalog.base(&base_id).is_none() {
            return Err(LayoutCreationError::BaseNotFound { base_id });
        }

        Ok(Self {
            catalog,
            base_id,
            instances: BTreeMap::new(),
        })
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn base_id(&self) -> &BaseId {
        &self.base_id
    }

    pub fn base_definition(&self) -> &BaseDefinition {
        self.catalog
            .base(&self.base_id)
            .expect("layout base ID must exist in its catalog")
    }

    pub fn bounds(&self) -> GridSize {
        self.base_definition().bounds()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn instance(&self, id: EntityId) -> Option<&BlockInstance> {
        self.instances.get(&id)
    }

    pub fn resolved_instance(&self, id: EntityId) -> Option<ResolvedInstance<'_>> {
        let instance = self.instance(id)?;
        let definition = self.catalog.buildable(instance.buildable_id())?;
        Some(ResolvedInstance {
            instance,
            definition,
            effective_footprint: instance.rotation().apply_to(definition.footprint()),
        })
    }

    pub fn instance_at(&self, point: GridPoint) -> Option<&BlockInstance> {
        self.instances
            .values()
            .find(|instance| self.occupied_rect(instance).contains(point))
    }

    pub fn instances(&self) -> impl Iterator<Item = &BlockInstance> {
        self.instances.values()
    }

    pub fn selection_rotation_pivot(
        &self,
        ids: &[EntityId],
    ) -> Result<Option<GridPoint>, InstanceEditError> {
        let (_, instances) = self.resolve_instances(ids)?;

        if instances.len() < 2 {
            return Ok(None);
        }

        let mut bounds = self.occupied_rect(&instances[0]);
        for instance in &instances[1..] {
            bounds = bounds.union(self.occupied_rect(instance));
        }

        Ok(Some(bounds.center_toward_top_left()))
    }

    pub fn remove_instance(&mut self, id: EntityId) -> Option<BlockInstance> {
        self.instances.remove(&id)
    }

    pub fn set_production_target(
        &mut self,
        id: EntityId,
        target: Option<ProductId>,
    ) -> Result<(), ProductionTargetError> {
        let instance = self
            .instances
            .get(&id)
            .ok_or(ProductionTargetError::EntityNotFound { id })?;
        let definition = self
            .catalog
            .buildable(instance.buildable_id())
            .expect("stored buildable ID must exist in the layout catalog");
        self.validate_production_target(definition, target.as_ref())
            .map_err(|error| match error {
                ProductionTargetValidationError::ProductNotFound { product_id } => {
                    ProductionTargetError::ProductNotFound { product_id }
                }
                ProductionTargetValidationError::UnsupportedProduct {
                    buildable_id,
                    product_id,
                } => ProductionTargetError::UnsupportedProduct {
                    buildable_id,
                    product_id,
                },
            })?;
        self.instances
            .get_mut(&id)
            .expect("validated entity ID must remain present")
            .production_target = target;
        Ok(())
    }

    pub fn move_instance(
        &mut self,
        id: EntityId,
        new_origin: GridPoint,
    ) -> Result<(), InstanceEditError> {
        let current = self
            .instances
            .get(&id)
            .cloned()
            .ok_or(InstanceEditError::EntityNotFound { id })?;
        let candidate = current.transformed(new_origin, current.rotation());

        self.replace_validated_instance(candidate)
    }

    pub fn rotate_instance(
        &mut self,
        id: EntityId,
        new_rotation: Rotation,
    ) -> Result<(), InstanceEditError> {
        let current = self
            .instances
            .get(&id)
            .cloned()
            .ok_or(InstanceEditError::EntityNotFound { id })?;
        let candidate = current.transformed(current.origin(), new_rotation);

        self.replace_validated_instance(candidate)
    }

    pub fn move_instances_by(
        &mut self,
        ids: &[EntityId],
        delta: GridPoint,
    ) -> Result<(), InstanceEditError> {
        self.replace_instances_atomically(ids, |instance| {
            let id = instance.id();
            let origin = instance.origin();
            let (Some(x), Some(y)) = (origin.x.checked_add(delta.x), origin.y.checked_add(delta.y))
            else {
                return Err(InstanceEditError::OutOfBounds { id });
            };
            Ok(instance.transformed(GridPoint::new(x, y), instance.rotation()))
        })
    }

    pub fn rotate_instances_clockwise_about(
        &mut self,
        ids: &[EntityId],
        pivot: GridPoint,
    ) -> Result<(), InstanceEditError> {
        let catalog = self.catalog.clone();
        self.replace_instances_atomically(ids, move |instance| {
            let id = instance.id();
            let footprint = catalog
                .buildable(instance.buildable_id())
                .expect("stored buildable ID must exist in the layout catalog")
                .footprint();
            let occupied = OccupiedRect::from_instance(&instance, footprint);
            let pivot_x = i64::from(pivot.x);
            let pivot_y = i64::from(pivot.y);
            let new_left = pivot_x + pivot_y - occupied.bottom;
            let new_top = pivot_y - pivot_x + occupied.left;
            let x = i32::try_from(new_left).map_err(|_| InstanceEditError::OutOfBounds { id })?;
            let y = i32::try_from(new_top).map_err(|_| InstanceEditError::OutOfBounds { id })?;

            Ok(instance.transformed(GridPoint::new(x, y), instance.rotation().clockwise()))
        })
    }

    fn replace_instances_atomically(
        &mut self,
        ids: &[EntityId],
        transform: impl Fn(BlockInstance) -> Result<BlockInstance, InstanceEditError>,
    ) -> Result<(), InstanceEditError> {
        let (ids, originals) = self.resolve_instances(ids)?;

        let mut candidate_layout = self.clone();
        for id in &ids {
            candidate_layout.instances.remove(id);
        }
        for original in originals {
            let candidate = transform(original)?;
            let id = candidate.id();
            candidate_layout
                .validate_spatial(&candidate, None)
                .map_err(|error| match error {
                    SpatialValidationError::OutOfBounds => InstanceEditError::OutOfBounds { id },
                    SpatialValidationError::Collision { conflicting_id } => {
                        InstanceEditError::Collision { id, conflicting_id }
                    }
                })?;
            candidate_layout.instances.insert(id, candidate);
        }

        *self = candidate_layout;
        Ok(())
    }

    fn resolve_instances(
        &self,
        ids: &[EntityId],
    ) -> Result<(BTreeSet<EntityId>, Vec<BlockInstance>), InstanceEditError> {
        let ids: BTreeSet<_> = ids.iter().copied().collect();
        let instances = ids
            .iter()
            .map(|id| {
                self.instances
                    .get(id)
                    .cloned()
                    .ok_or(InstanceEditError::EntityNotFound { id: *id })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok((ids, instances))
    }

    pub fn place(&mut self, instance: BlockInstance) -> Result<(), PlacementError> {
        let id = instance.id();

        if self.instances.contains_key(&id) {
            return Err(PlacementError::DuplicateEntityId { id });
        }

        let definition = self
            .catalog
            .buildable(instance.buildable_id())
            .ok_or_else(|| PlacementError::BuildableNotFound {
                id,
                buildable_id: instance.buildable_id().clone(),
            })?;
        self.validate_production_target(definition, instance.production_target())
            .map_err(|error| match error {
                ProductionTargetValidationError::ProductNotFound { product_id } => {
                    PlacementError::ProductNotFound { id, product_id }
                }
                ProductionTargetValidationError::UnsupportedProduct {
                    buildable_id,
                    product_id,
                } => PlacementError::UnsupportedProduct {
                    id,
                    buildable_id,
                    product_id,
                },
            })?;

        self.validate_spatial(&instance, None)
            .map_err(|error| match error {
                SpatialValidationError::OutOfBounds => PlacementError::OutOfBounds { id },
                SpatialValidationError::Collision { conflicting_id } => {
                    PlacementError::Collision { id, conflicting_id }
                }
            })?;

        self.instances.insert(id, instance);
        Ok(())
    }

    fn validate_production_target(
        &self,
        definition: &BuildableDefinition,
        target: Option<&ProductId>,
    ) -> Result<(), ProductionTargetValidationError> {
        let Some(product_id) = target else {
            return Ok(());
        };
        if self.catalog.product(product_id).is_none() {
            return Err(ProductionTargetValidationError::ProductNotFound {
                product_id: product_id.clone(),
            });
        }
        if !definition.production_targets().contains(product_id) {
            return Err(ProductionTargetValidationError::UnsupportedProduct {
                buildable_id: definition.id().clone(),
                product_id: product_id.clone(),
            });
        }

        Ok(())
    }

    fn replace_validated_instance(
        &mut self,
        candidate: BlockInstance,
    ) -> Result<(), InstanceEditError> {
        let id = candidate.id();

        self.validate_spatial(&candidate, Some(id))
            .map_err(|error| match error {
                SpatialValidationError::OutOfBounds => InstanceEditError::OutOfBounds { id },
                SpatialValidationError::Collision { conflicting_id } => {
                    InstanceEditError::Collision { id, conflicting_id }
                }
            })?;

        self.instances.insert(id, candidate);
        Ok(())
    }

    fn validate_spatial(
        &self,
        candidate: &BlockInstance,
        ignored_id: Option<EntityId>,
    ) -> Result<(), SpatialValidationError> {
        let occupied_rect = self.occupied_rect(candidate);

        if !occupied_rect.is_within(self.bounds()) {
            return Err(SpatialValidationError::OutOfBounds);
        }

        if let Some((&conflicting_id, _)) = self.instances.iter().find(|(existing_id, existing)| {
            Some(**existing_id) != ignored_id
                && occupied_rect.overlaps(self.occupied_rect(existing))
        }) {
            return Err(SpatialValidationError::Collision { conflicting_id });
        }

        Ok(())
    }

    fn buildable_definition(&self, instance: &BlockInstance) -> &BuildableDefinition {
        self.catalog
            .buildable(instance.buildable_id())
            .expect("stored buildable ID must exist in the layout catalog")
    }

    fn occupied_rect(&self, instance: &BlockInstance) -> OccupiedRect {
        OccupiedRect::from_instance(instance, self.buildable_definition(instance).footprint())
    }
}
