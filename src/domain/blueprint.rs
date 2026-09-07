use std::sync::Arc;

use super::catalog::{BuildableId, Catalog, ProductId};
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

    /// Reconstructs a `Blueprint` from already-decoded, already-canonicalized nodes
    /// (for example, from a persisted document). Every node's `buildable_id` and, if
    /// present, `production_target` are validated against `catalog` exactly like a
    /// live domain mutation would validate them; nothing is trusted from the input
    /// beyond that. Canonical local-ID numbering and node ordering are the caller's
    /// responsibility (the persistence layer enforces `1..=N` before calling this).
    pub fn from_nodes(
        id: BlueprintId,
        catalog: Catalog,
        metadata: DocumentMetadata,
        nodes: Vec<BlueprintNodeInput>,
    ) -> Result<Self, BlueprintNodeValidationError> {
        if nodes.is_empty() {
            return Err(BlueprintNodeValidationError::EmptyNodes);
        }

        let nodes = nodes
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

        Ok(Self {
            id,
            provenance: CatalogProvenance::from_catalog(&catalog),
            metadata,
            nodes,
        })
    }
}
