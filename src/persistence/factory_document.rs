use crate::domain::catalog::{BaseId, BuildableId, Catalog, CatalogId, ProductId};
use crate::domain::document::{CatalogProvenance, DocumentMetadata, DocumentMetadataError};
use crate::domain::geometry::{GridPoint, Rotation};
use crate::domain::layout::{BlockInstance, EntityId, FactoryLayout, PlacementError};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const FACTORY_DOCUMENT_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogCompatibility {
    Exact,
    CatalogIdMismatch,
    DataVersionMismatch,
    CatalogAndDataVersionMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFactoryDocument {
    pub layout: FactoryLayout,
    pub next_entity_id: Option<u64>,
    pub metadata: DocumentMetadata,
    pub compatibility: CatalogCompatibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactoryLayoutErrorKind {
    DuplicateEntityId,
    BuildableNotFound,
    ProductNotFound,
    UnsupportedProduct,
    OutOfBounds,
    Collision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactoryDocumentError {
    InvalidJson {
        line: usize,
        column: usize,
    },
    UnsupportedSchemaVersion,
    InvalidCatalogId,
    InvalidCatalogDataVersion,
    InvalidMetadata(DocumentMetadataError),
    InvalidTimestamp,
    InvalidBaseId,
    InvalidEntityId {
        entity_index: usize,
    },
    InvalidBuildableId {
        entity_index: usize,
    },
    InvalidProductionTarget {
        entity_index: usize,
    },
    InvalidRotation {
        entity_index: usize,
    },
    InvalidNextEntityId,
    InvalidLayout {
        entity_index: usize,
        kind: FactoryLayoutErrorKind,
    },
    Serialization,
}

impl fmt::Display for FactoryDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { line, column } => {
                write!(
                    formatter,
                    "factory document JSON is invalid at line {line}, column {column}"
                )
            }
            Self::UnsupportedSchemaVersion => {
                formatter.write_str("factory document schema version is not supported")
            }
            Self::InvalidCatalogId => formatter.write_str("factory document catalog ID is invalid"),
            Self::InvalidCatalogDataVersion => {
                formatter.write_str("factory document catalog data version is invalid")
            }
            Self::InvalidMetadata(error) => {
                write!(formatter, "factory document metadata is invalid: {error}")
            }
            Self::InvalidTimestamp => formatter.write_str("factory document timestamp is invalid"),
            Self::InvalidBaseId => formatter.write_str("factory document base ID is invalid"),
            Self::InvalidEntityId { entity_index } => write!(
                formatter,
                "factory document entity {} has an invalid ID",
                entity_index + 1
            ),
            Self::InvalidBuildableId { entity_index } => write!(
                formatter,
                "factory document entity {} has an invalid buildable ID",
                entity_index + 1
            ),
            Self::InvalidProductionTarget { entity_index } => write!(
                formatter,
                "factory document entity {} has an invalid production target",
                entity_index + 1
            ),
            Self::InvalidRotation { entity_index } => write!(
                formatter,
                "factory document entity {} has an invalid rotation",
                entity_index + 1
            ),
            Self::InvalidNextEntityId => {
                formatter.write_str("factory document next entity ID is invalid")
            }
            Self::InvalidLayout {
                entity_index,
                kind: FactoryLayoutErrorKind::DuplicateEntityId,
            } => write!(
                formatter,
                "factory document entity {} duplicates another entity ID",
                entity_index + 1
            ),
            Self::InvalidLayout {
                entity_index,
                kind: FactoryLayoutErrorKind::BuildableNotFound,
            } => write!(
                formatter,
                "factory document entity {} references an unavailable buildable",
                entity_index + 1
            ),
            Self::InvalidLayout {
                entity_index,
                kind: FactoryLayoutErrorKind::ProductNotFound,
            } => write!(
                formatter,
                "factory document entity {} references an unavailable product",
                entity_index + 1
            ),
            Self::InvalidLayout {
                entity_index,
                kind: FactoryLayoutErrorKind::UnsupportedProduct,
            } => write!(
                formatter,
                "factory document entity {} selects a product unsupported by its buildable",
                entity_index + 1
            ),
            Self::InvalidLayout {
                entity_index,
                kind: FactoryLayoutErrorKind::OutOfBounds,
            } => write!(
                formatter,
                "factory document entity {} extends outside the selected base",
                entity_index + 1
            ),
            Self::InvalidLayout {
                entity_index,
                kind: FactoryLayoutErrorKind::Collision,
            } => write!(
                formatter,
                "factory document entity {} overlaps another entity",
                entity_index + 1
            ),
            Self::Serialization => formatter.write_str("factory document could not be serialized"),
        }
    }
}

impl std::error::Error for FactoryDocumentError {}

#[derive(Debug, Deserialize)]
struct DocumentHeader {
    schema_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactoryDocumentV1Dto {
    schema_version: u64,
    catalog_id: String,
    catalog_data_version: String,
    metadata: DocumentMetadataDto,
    base_id: String,
    next_entity_id: RequiredNullable<u64>,
    entities: Vec<FactoryEntityDto>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DocumentMetadataDto {
    name: String,
    description: RequiredNullable<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactoryEntityDto {
    id: u64,
    buildable_id: String,
    origin: GridPointDto,
    rotation_degrees: u16,
    production_target: RequiredNullable<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RequiredNullable<T> {
    Value(T),
    Null,
}

impl<T> RequiredNullable<T> {
    fn from_option(value: Option<T>) -> Self {
        value.map_or(Self::Null, Self::Value)
    }

    fn into_option(self) -> Option<T> {
        match self {
            Self::Value(value) => Some(value),
            Self::Null => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GridPointDto {
    x: i32,
    y: i32,
}

pub fn encode_factory_document(
    layout: &FactoryLayout,
    next_entity_id: Option<u64>,
    metadata: &DocumentMetadata,
) -> Result<Vec<u8>, FactoryDocumentError> {
    validate_next_entity_id(layout, next_entity_id)?;
    let provenance = CatalogProvenance::from_catalog(layout.catalog());
    let metadata = DocumentMetadataDto {
        name: metadata.name().to_owned(),
        description: RequiredNullable::from_option(metadata.description().map(str::to_owned)),
        created_at: format_timestamp(metadata.created_at())?,
        updated_at: format_timestamp(metadata.updated_at())?,
    };
    let entities = layout
        .instances()
        .enumerate()
        .map(|(entity_index, instance)| {
            if instance.id().value() == 0 {
                return Err(FactoryDocumentError::InvalidEntityId { entity_index });
            }
            Ok(FactoryEntityDto {
                id: instance.id().value(),
                buildable_id: instance.buildable_id().as_str().to_owned(),
                origin: GridPointDto {
                    x: instance.origin().x,
                    y: instance.origin().y,
                },
                rotation_degrees: rotation_degrees(instance.rotation()),
                production_target: RequiredNullable::from_option(
                    instance
                        .production_target()
                        .map(|id| id.as_str().to_owned()),
                ),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let dto = FactoryDocumentV1Dto {
        schema_version: FACTORY_DOCUMENT_SCHEMA_VERSION,
        catalog_id: provenance.catalog_id().as_str().to_owned(),
        catalog_data_version: provenance.data_version().to_string(),
        metadata,
        base_id: layout.base_id().as_str().to_owned(),
        next_entity_id: RequiredNullable::from_option(next_entity_id),
        entities,
    };
    let mut encoded =
        serde_json::to_vec_pretty(&dto).map_err(|_| FactoryDocumentError::Serialization)?;
    encoded.push(b'\n');
    Ok(encoded)
}

pub fn decode_factory_document(
    bytes: &[u8],
    active_catalog: Catalog,
) -> Result<LoadedFactoryDocument, FactoryDocumentError> {
    let header: DocumentHeader = decode_json(bytes)?;
    match header.schema_version {
        FACTORY_DOCUMENT_SCHEMA_VERSION => decode_factory_document_v1(bytes, active_catalog),
        _ => Err(FactoryDocumentError::UnsupportedSchemaVersion),
    }
}

fn decode_factory_document_v1(
    bytes: &[u8],
    active_catalog: Catalog,
) -> Result<LoadedFactoryDocument, FactoryDocumentError> {
    let dto: FactoryDocumentV1Dto = decode_json(bytes)?;
    let document_catalog_id =
        CatalogId::new(dto.catalog_id).map_err(|_| FactoryDocumentError::InvalidCatalogId)?;
    let document_data_version = Version::parse(&dto.catalog_data_version)
        .map_err(|_| FactoryDocumentError::InvalidCatalogDataVersion)?;
    let compatibility = compatibility(
        &document_catalog_id,
        &document_data_version,
        &active_catalog,
    );
    let created_at = parse_timestamp(&dto.metadata.created_at)?;
    let updated_at = parse_timestamp(&dto.metadata.updated_at)?;
    let metadata = DocumentMetadata::new(
        &dto.metadata.name,
        dto.metadata.description.into_option().as_deref(),
        created_at,
        updated_at,
    )
    .map_err(FactoryDocumentError::InvalidMetadata)?;
    let base_id = BaseId::new(dto.base_id).map_err(|_| FactoryDocumentError::InvalidBaseId)?;
    let mut layout = FactoryLayout::new(active_catalog, base_id)
        .map_err(|_| FactoryDocumentError::InvalidBaseId)?;
    let next_entity_id = dto.next_entity_id.into_option();

    let mut entities = dto.entities.into_iter().enumerate().collect::<Vec<_>>();
    entities.sort_by_key(|(_, entity)| entity.id);
    for (entity_index, entity) in entities {
        if entity.id == 0 {
            return Err(FactoryDocumentError::InvalidEntityId { entity_index });
        }
        let buildable_id = BuildableId::new(entity.buildable_id)
            .map_err(|_| FactoryDocumentError::InvalidBuildableId { entity_index })?;
        let production_target = entity
            .production_target
            .into_option()
            .map(ProductId::new)
            .transpose()
            .map_err(|_| FactoryDocumentError::InvalidProductionTarget { entity_index })?;
        let rotation = rotation_from_degrees(entity.rotation_degrees)
            .ok_or(FactoryDocumentError::InvalidRotation { entity_index })?;
        let instance = BlockInstance::new(
            EntityId::new(entity.id),
            buildable_id,
            GridPoint::new(entity.origin.x, entity.origin.y),
            rotation,
        )
        .with_production_target(production_target);
        layout
            .place(instance)
            .map_err(|error| FactoryDocumentError::InvalidLayout {
                entity_index,
                kind: layout_error_kind(&error),
            })?;
    }
    validate_next_entity_id(&layout, next_entity_id)?;

    Ok(LoadedFactoryDocument {
        layout,
        next_entity_id,
        metadata,
        compatibility,
    })
}

fn decode_json<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, FactoryDocumentError> {
    serde_json::from_slice(bytes).map_err(|error| FactoryDocumentError::InvalidJson {
        line: error.line(),
        column: error.column(),
    })
}

fn format_timestamp(value: OffsetDateTime) -> Result<String, FactoryDocumentError> {
    value
        .format(&Rfc3339)
        .map_err(|_| FactoryDocumentError::InvalidTimestamp)
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime, FactoryDocumentError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| FactoryDocumentError::InvalidTimestamp)
}

fn rotation_degrees(rotation: Rotation) -> u16 {
    match rotation {
        Rotation::Zero => 0,
        Rotation::Clockwise90 => 90,
        Rotation::Clockwise180 => 180,
        Rotation::Clockwise270 => 270,
    }
}

fn rotation_from_degrees(degrees: u16) -> Option<Rotation> {
    match degrees {
        0 => Some(Rotation::Zero),
        90 => Some(Rotation::Clockwise90),
        180 => Some(Rotation::Clockwise180),
        270 => Some(Rotation::Clockwise270),
        _ => None,
    }
}

fn compatibility(
    document_catalog_id: &CatalogId,
    document_data_version: &Version,
    active_catalog: &Catalog,
) -> CatalogCompatibility {
    let id_matches = document_catalog_id == active_catalog.metadata().catalog_id();
    let version_matches = document_data_version == active_catalog.metadata().data_version();
    match (id_matches, version_matches) {
        (true, true) => CatalogCompatibility::Exact,
        (false, true) => CatalogCompatibility::CatalogIdMismatch,
        (true, false) => CatalogCompatibility::DataVersionMismatch,
        (false, false) => CatalogCompatibility::CatalogAndDataVersionMismatch,
    }
}

fn validate_next_entity_id(
    layout: &FactoryLayout,
    next_entity_id: Option<u64>,
) -> Result<(), FactoryDocumentError> {
    let Some(next_entity_id) = next_entity_id else {
        return Ok(());
    };
    if next_entity_id == 0
        || layout
            .instances()
            .any(|instance| instance.id().value() >= next_entity_id)
    {
        return Err(FactoryDocumentError::InvalidNextEntityId);
    }
    Ok(())
}

fn layout_error_kind(error: &PlacementError) -> FactoryLayoutErrorKind {
    match error {
        PlacementError::DuplicateEntityId { .. } => FactoryLayoutErrorKind::DuplicateEntityId,
        PlacementError::BuildableNotFound { .. } => FactoryLayoutErrorKind::BuildableNotFound,
        PlacementError::ProductNotFound { .. } => FactoryLayoutErrorKind::ProductNotFound,
        PlacementError::UnsupportedProduct { .. } => FactoryLayoutErrorKind::UnsupportedProduct,
        PlacementError::OutOfBounds { .. } => FactoryLayoutErrorKind::OutOfBounds,
        PlacementError::Collision { .. } => FactoryLayoutErrorKind::Collision,
    }
}
