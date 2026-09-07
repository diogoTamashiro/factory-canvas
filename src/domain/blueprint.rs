use std::sync::Arc;

use super::catalog::{BuildableId, ProductId};
use super::document::{CatalogProvenance, DocumentMetadata};
use super::geometry::{GridPoint, Rotation};
use super::layout::{EntityId, FactoryLayout};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintCreationError {
    EmptySelection,
    EntityNotFound { id: EntityId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blueprint {
    id: BlueprintId,
    provenance: CatalogProvenance,
    metadata: DocumentMetadata,
    nodes: Vec<BlueprintNode>,
}

impl Blueprint {
    pub fn from_selection(
        layout: &FactoryLayout,
        selected_ids: impl IntoIterator<Item = EntityId>,
        id: BlueprintId,
        metadata: DocumentMetadata,
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

        let nodes = instances
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

        Ok(Self {
            id,
            provenance: CatalogProvenance::from_catalog(layout.catalog()),
            metadata,
            nodes,
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
}
