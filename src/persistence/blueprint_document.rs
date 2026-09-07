use crate::domain::blueprint::{
    Blueprint, BlueprintEntityId, BlueprintId, BlueprintIdError, BlueprintNodeInput,
    BlueprintNodeValidationError,
};
use crate::domain::catalog::{BuildableId, Catalog, CatalogId, ProductId};
use crate::domain::document::{DocumentMetadata, DocumentMetadataError};
use crate::domain::geometry::{GridPoint, Rotation};
use crate::persistence::factory_document::CatalogCompatibility;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const BLUEPRINT_DOCUMENT_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedBlueprintDocument {
    pub blueprint: Blueprint,
    pub compatibility: CatalogCompatibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueprintNodeErrorKind {
    BuildableNotFound,
    ProductNotFound,
    UnsupportedProduct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintDocumentError {
    InvalidJson {
        line: usize,
        column: usize,
    },
    UnsupportedSchemaVersion,
    InvalidCatalogId,
    InvalidCatalogDataVersion,
    InvalidBlueprintId,
    InvalidMetadata(DocumentMetadataError),
    InvalidTimestamp,
    NonCanonicalNodeId {
        node_index: usize,
    },
    InvalidBuildableId {
        node_index: usize,
    },
    InvalidProductionTarget {
        node_index: usize,
    },
    InvalidRotation {
        node_index: usize,
    },
    InvalidNode {
        node_index: usize,
        kind: BlueprintNodeErrorKind,
    },
    EmptyNodes,
    Serialization,
}

impl fmt::Display for BlueprintDocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { line, column } => write!(
                formatter,
                "blueprint document JSON is invalid at line {line}, column {column}"
            ),
            Self::UnsupportedSchemaVersion => {
                formatter.write_str("blueprint document schema version is not supported")
            }
            Self::InvalidCatalogId => {
                formatter.write_str("blueprint document catalog ID is invalid")
            }
            Self::InvalidCatalogDataVersion => {
                formatter.write_str("blueprint document catalog data version is invalid")
            }
            Self::InvalidBlueprintId => {
                formatter.write_str("blueprint document blueprint ID is invalid")
            }
            Self::InvalidMetadata(error) => {
                write!(formatter, "blueprint document metadata is invalid: {error}")
            }
            Self::InvalidTimestamp => {
                formatter.write_str("blueprint document timestamp is invalid")
            }
            Self::NonCanonicalNodeId { node_index } => write!(
                formatter,
                "blueprint document node {} has a non-canonical local ID",
                node_index + 1
            ),
            Self::InvalidBuildableId { node_index } => write!(
                formatter,
                "blueprint document node {} has an invalid buildable ID",
                node_index + 1
            ),
            Self::InvalidProductionTarget { node_index } => write!(
                formatter,
                "blueprint document node {} has an invalid production target",
                node_index + 1
            ),
            Self::InvalidRotation { node_index } => write!(
                formatter,
                "blueprint document node {} has an invalid rotation",
                node_index + 1
            ),
            Self::InvalidNode {
                node_index,
                kind: BlueprintNodeErrorKind::BuildableNotFound,
            } => write!(
                formatter,
                "blueprint document node {} references an unavailable buildable",
                node_index + 1
            ),
            Self::InvalidNode {
                node_index,
                kind: BlueprintNodeErrorKind::ProductNotFound,
            } => write!(
                formatter,
                "blueprint document node {} references an unavailable product",
                node_index + 1
            ),
            Self::InvalidNode {
                node_index,
                kind: BlueprintNodeErrorKind::UnsupportedProduct,
            } => write!(
                formatter,
                "blueprint document node {} selects a product unsupported by its buildable",
                node_index + 1
            ),
            Self::EmptyNodes => formatter.write_str("blueprint document has no nodes"),
            Self::Serialization => {
                formatter.write_str("blueprint document could not be serialized")
            }
        }
    }
}

impl std::error::Error for BlueprintDocumentError {}

#[derive(Debug, Deserialize)]
struct DocumentHeader {
    schema_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BlueprintDocumentV1Dto {
    schema_version: u64,
    catalog_id: String,
    catalog_data_version: String,
    blueprint_id: String,
    metadata: DocumentMetadataDto,
    nodes: Vec<BlueprintNodeDto>,
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
struct BlueprintNodeDto {
    id: u64,
    buildable_id: String,
    relative_origin: GridPointDto,
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

pub fn encode_blueprint_document(blueprint: &Blueprint) -> Result<Vec<u8>, BlueprintDocumentError> {
    encode_blueprint_document_as(blueprint, blueprint.id())
}

/// Encodes `blueprint` exactly like [`encode_blueprint_document`], except the
/// document's `blueprint_id` field is `id` rather than `blueprint.id()`.
///
/// This exists solely for the blueprint library's save-time ID-collision
/// retry (`crate::persistence::blueprint_library`), which must persist a
/// blueprint's data under a freshly generated identifier without
/// reconstructing a new domain `Blueprint` value — identity is not a
/// catalog-validated property, so no `Catalog` should be required just to
/// change it. Crate-private: this is an internal persistence detail, not
/// part of the document codec's public contract.
pub(crate) fn encode_blueprint_document_as(
    blueprint: &Blueprint,
    id: &BlueprintId,
) -> Result<Vec<u8>, BlueprintDocumentError> {
    let metadata = blueprint.metadata();
    let metadata_dto = DocumentMetadataDto {
        name: metadata.name().to_owned(),
        description: RequiredNullable::from_option(metadata.description().map(str::to_owned)),
        created_at: format_timestamp(metadata.created_at())?,
        updated_at: format_timestamp(metadata.updated_at())?,
    };
    let nodes = blueprint
        .nodes()
        .iter()
        .map(|node| BlueprintNodeDto {
            id: node.id().value(),
            buildable_id: node.buildable_id().as_str().to_owned(),
            relative_origin: GridPointDto {
                x: node.relative_origin().x,
                y: node.relative_origin().y,
            },
            rotation_degrees: rotation_degrees(node.rotation()),
            production_target: RequiredNullable::from_option(
                node.production_target().map(|id| id.as_str().to_owned()),
            ),
        })
        .collect();
    let dto = BlueprintDocumentV1Dto {
        schema_version: BLUEPRINT_DOCUMENT_SCHEMA_VERSION,
        catalog_id: blueprint.provenance().catalog_id().as_str().to_owned(),
        catalog_data_version: blueprint.provenance().data_version().to_string(),
        blueprint_id: id.as_str().to_owned(),
        metadata: metadata_dto,
        nodes,
    };
    let mut encoded =
        serde_json::to_vec_pretty(&dto).map_err(|_| BlueprintDocumentError::Serialization)?;
    encoded.push(b'\n');
    Ok(encoded)
}

pub fn decode_blueprint_document(
    bytes: &[u8],
    active_catalog: Catalog,
) -> Result<LoadedBlueprintDocument, BlueprintDocumentError> {
    let header: DocumentHeader = decode_json(bytes)?;
    match header.schema_version {
        BLUEPRINT_DOCUMENT_SCHEMA_VERSION => decode_blueprint_document_v1(bytes, active_catalog),
        _ => Err(BlueprintDocumentError::UnsupportedSchemaVersion),
    }
}

fn decode_blueprint_document_v1(
    bytes: &[u8],
    active_catalog: Catalog,
) -> Result<LoadedBlueprintDocument, BlueprintDocumentError> {
    let dto: BlueprintDocumentV1Dto = decode_json(bytes)?;
    let document_catalog_id =
        CatalogId::new(dto.catalog_id).map_err(|_| BlueprintDocumentError::InvalidCatalogId)?;
    let document_data_version = Version::parse(&dto.catalog_data_version)
        .map_err(|_| BlueprintDocumentError::InvalidCatalogDataVersion)?;
    let compatibility = compatibility(
        &document_catalog_id,
        &document_data_version,
        &active_catalog,
    );
    let blueprint_id = BlueprintId::parse(&dto.blueprint_id)
        .map_err(|_: BlueprintIdError| BlueprintDocumentError::InvalidBlueprintId)?;
    let created_at = parse_timestamp(&dto.metadata.created_at)?;
    let updated_at = parse_timestamp(&dto.metadata.updated_at)?;
    let metadata = DocumentMetadata::new(
        &dto.metadata.name,
        dto.metadata.description.into_option().as_deref(),
        created_at,
        updated_at,
    )
    .map_err(BlueprintDocumentError::InvalidMetadata)?;

    if dto.nodes.is_empty() {
        return Err(BlueprintDocumentError::EmptyNodes);
    }

    let mut sorted_nodes = dto.nodes.into_iter().enumerate().collect::<Vec<_>>();
    sorted_nodes.sort_by_key(|(_, node)| node.id);
    let mut inputs = Vec::with_capacity(sorted_nodes.len());
    for (expected_id, (node_index, node)) in (1u64..).zip(sorted_nodes) {
        if node.id != expected_id {
            return Err(BlueprintDocumentError::NonCanonicalNodeId { node_index });
        }
        let buildable_id = BuildableId::new(node.buildable_id)
            .map_err(|_| BlueprintDocumentError::InvalidBuildableId { node_index })?;
        let production_target = node
            .production_target
            .into_option()
            .map(ProductId::new)
            .transpose()
            .map_err(|_| BlueprintDocumentError::InvalidProductionTarget { node_index })?;
        let rotation = rotation_from_degrees(node.rotation_degrees)
            .ok_or(BlueprintDocumentError::InvalidRotation { node_index })?;
        inputs.push((
            node_index,
            BlueprintNodeInput {
                id: BlueprintEntityId::new(expected_id),
                buildable_id,
                relative_origin: GridPoint::new(node.relative_origin.x, node.relative_origin.y),
                rotation,
                production_target,
            },
        ));
    }
    let node_indices_by_local_id: Vec<usize> = inputs.iter().map(|(index, _)| *index).collect();
    let inputs = inputs.into_iter().map(|(_, input)| input).collect();

    let blueprint = Blueprint::from_nodes(blueprint_id, active_catalog, metadata, inputs)
        .map_err(|error| blueprint_node_validation_error(error, &node_indices_by_local_id))?;

    Ok(LoadedBlueprintDocument {
        blueprint,
        compatibility,
    })
}

fn blueprint_node_validation_error(
    error: BlueprintNodeValidationError,
    node_indices_by_local_id: &[usize],
) -> BlueprintDocumentError {
    let node_index_for =
        |node_id: BlueprintEntityId| node_indices_by_local_id[(node_id.value() - 1) as usize];
    match error {
        BlueprintNodeValidationError::EmptyNodes => BlueprintDocumentError::EmptyNodes,
        BlueprintNodeValidationError::BuildableNotFound { node_id, .. } => {
            BlueprintDocumentError::InvalidNode {
                node_index: node_index_for(node_id),
                kind: BlueprintNodeErrorKind::BuildableNotFound,
            }
        }
        BlueprintNodeValidationError::ProductNotFound { node_id, .. } => {
            BlueprintDocumentError::InvalidNode {
                node_index: node_index_for(node_id),
                kind: BlueprintNodeErrorKind::ProductNotFound,
            }
        }
        BlueprintNodeValidationError::UnsupportedProduct { node_id, .. } => {
            BlueprintDocumentError::InvalidNode {
                node_index: node_index_for(node_id),
                kind: BlueprintNodeErrorKind::UnsupportedProduct,
            }
        }
    }
}

fn decode_json<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, BlueprintDocumentError> {
    serde_json::from_slice(bytes).map_err(|error| BlueprintDocumentError::InvalidJson {
        line: error.line(),
        column: error.column(),
    })
}

fn format_timestamp(value: OffsetDateTime) -> Result<String, BlueprintDocumentError> {
    value
        .format(&Rfc3339)
        .map_err(|_| BlueprintDocumentError::InvalidTimestamp)
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime, BlueprintDocumentError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| BlueprintDocumentError::InvalidTimestamp)
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
