use super::base::BaseTemplate;
use super::catalog::BlockTemplate;
use super::geometry::{GridPoint, GridSize, Rotation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactoryLayout {
    base_template: BaseTemplate,
}

impl FactoryLayout {
    pub const fn new(base_template: BaseTemplate) -> Self {
        Self { base_template }
    }

    pub const fn base_template(&self) -> BaseTemplate {
        self.base_template
    }

    pub fn bounds(&self) -> GridSize {
        self.base_template.bounds()
    }
}
