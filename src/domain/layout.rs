use std::collections::BTreeMap;

use super::base::BaseTemplate;
use super::catalog::BlockTemplate;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockInstance {
    id: EntityId,
    template: BlockTemplate,
    origin: GridPoint,
    rotation: Rotation,
}

impl BlockInstance {
    pub const fn new(
        id: EntityId,
        template: BlockTemplate,
        origin: GridPoint,
        rotation: Rotation,
    ) -> Self {
        Self {
            id,
            template,
            origin,
            rotation,
        }
    }

    pub const fn id(self) -> EntityId {
        self.id
    }

    pub const fn template(self) -> BlockTemplate {
        self.template
    }

    pub const fn origin(self) -> GridPoint {
        self.origin
    }

    pub const fn rotation(self) -> Rotation {
        self.rotation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementError {
    DuplicateEntityId {
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
    fn from_instance(instance: BlockInstance) -> Self {
        let origin = instance.origin();
        let footprint = instance
            .rotation()
            .apply_to(instance.template().definition().footprint());
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactoryLayout {
    base_template: BaseTemplate,
    instances: BTreeMap<EntityId, BlockInstance>,
}

impl FactoryLayout {
    pub const fn new(base_template: BaseTemplate) -> Self {
        Self {
            base_template,
            instances: BTreeMap::new(),
        }
    }

    pub const fn base_template(&self) -> BaseTemplate {
        self.base_template
    }

    pub fn bounds(&self) -> GridSize {
        self.base_template.bounds()
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

    pub fn instance_at(&self, point: GridPoint) -> Option<&BlockInstance> {
        self.instances
            .values()
            .find(|instance| OccupiedRect::from_instance(**instance).contains(point))
    }

    pub fn instances(&self) -> impl Iterator<Item = &BlockInstance> {
        self.instances.values()
    }

    pub fn remove_instance(&mut self, id: EntityId) -> Option<BlockInstance> {
        self.instances.remove(&id)
    }

    pub fn move_instance(
        &mut self,
        id: EntityId,
        new_origin: GridPoint,
    ) -> Result<(), InstanceEditError> {
        let current = self
            .instances
            .get(&id)
            .copied()
            .ok_or(InstanceEditError::EntityNotFound { id })?;
        let candidate = BlockInstance::new(id, current.template(), new_origin, current.rotation());

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
            .copied()
            .ok_or(InstanceEditError::EntityNotFound { id })?;
        let candidate = BlockInstance::new(id, current.template(), current.origin(), new_rotation);

        self.replace_validated_instance(candidate)
    }

    pub fn place(&mut self, instance: BlockInstance) -> Result<(), PlacementError> {
        let id = instance.id();

        if self.instances.contains_key(&id) {
            return Err(PlacementError::DuplicateEntityId { id });
        }

        self.validate_spatial(instance, None)
            .map_err(|error| match error {
                SpatialValidationError::OutOfBounds => PlacementError::OutOfBounds { id },
                SpatialValidationError::Collision { conflicting_id } => {
                    PlacementError::Collision { id, conflicting_id }
                }
            })?;

        self.instances.insert(id, instance);
        Ok(())
    }

    fn replace_validated_instance(
        &mut self,
        candidate: BlockInstance,
    ) -> Result<(), InstanceEditError> {
        let id = candidate.id();

        self.validate_spatial(candidate, Some(id))
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
        candidate: BlockInstance,
        ignored_id: Option<EntityId>,
    ) -> Result<(), SpatialValidationError> {
        let occupied_rect = OccupiedRect::from_instance(candidate);

        if !occupied_rect.is_within(self.bounds()) {
            return Err(SpatialValidationError::OutOfBounds);
        }

        if let Some((&conflicting_id, _)) = self.instances.iter().find(|(existing_id, existing)| {
            Some(**existing_id) != ignored_id
                && occupied_rect.overlaps(OccupiedRect::from_instance(**existing))
        }) {
            return Err(SpatialValidationError::Collision { conflicting_id });
        }

        Ok(())
    }
}
