use super::colors::TEXT_MUTED;
use eframe::egui::Color32;
use factory_canvas::catalog_loader::CatalogLoadError;
use factory_canvas::domain::blueprint::BlueprintInsertionError;
use factory_canvas::domain::catalog::{BuildableDefinition, BuildableId, Catalog, ProductId};
use factory_canvas::domain::geometry::{GridPoint, Rotation};
use factory_canvas::domain::layout::{
    EntityId, InstanceEditError, PlacementError, ProductionTargetError,
};
use factory_canvas::persistence::blueprint_library::BlueprintLibrarySaveError;
use factory_canvas::persistence::factory_document::{CatalogCompatibility, FactoryDocumentError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum EditorNotice {
    SelectBlock,
    ReadyToPlace {
        buildable_id: BuildableId,
    },
    InstanceSelected {
        id: EntityId,
        buildable_id: BuildableId,
    },
    InstancesSelected {
        count: usize,
    },
    InstanceRemoved {
        id: EntityId,
        buildable_id: BuildableId,
    },
    InstancesRemoved {
        count: usize,
    },
    InstanceMoved {
        id: EntityId,
        origin: GridPoint,
    },
    InstancesMoved {
        count: usize,
    },
    InstanceRotated {
        id: EntityId,
        rotation: Rotation,
    },
    InstancesRotated {
        count: usize,
    },
    InstanceEditRejected(InstanceEditError),
    ProductionTargetChanged {
        id: EntityId,
        product_id: Option<ProductId>,
    },
    ProductionTargetRejected(ProductionTargetError),
    Placed {
        id: EntityId,
        buildable_id: BuildableId,
        origin: GridPoint,
    },
    PlacementRejected(PlacementError),
    EntityIdsExhausted,
    BaseChanged,
    DocumentSaved,
    DocumentOpened(CatalogCompatibility),
    DocumentOpenFailed(FactoryDocumentError),
    DocumentSaveFailed(FactoryDocumentError),
    BlueprintSaved,
    BlueprintSaveFailed(BlueprintLibrarySaveError),
    BlueprintArmedForInsertion,
    BlueprintInsertionUnavailable,
    BlueprintInserted {
        node_count: usize,
    },
    BlueprintInsertionRejected(BlueprintInsertionError),
    Undone,
    Redone,
}

fn buildable_for_id<'a>(id: &BuildableId, catalog: &'a Catalog) -> &'a BuildableDefinition {
    catalog
        .buildable(id)
        .expect("buildable ID from active catalog must resolve")
}

pub(super) fn safe_catalog_load_detail(error: &CatalogLoadError) -> String {
    match error {
        CatalogLoadError::ManifestRead(_) => {
            "The private catalog manifest could not be read.".to_owned()
        }
        CatalogLoadError::ModuleRead { module, .. } => {
            format!("The private {module} catalog module could not be read.")
        }
        CatalogLoadError::ModuleOutsideRoot { module } => format!(
            "The private {module} catalog module resolves outside the package root."
        ),
        CatalogLoadError::InvalidJson {
            module,
            kind,
            line,
            column,
        } => {
            let description = match kind {
                factory_canvas::catalog_loader::CatalogJsonErrorKind::Io => {
                    "could not be decoded"
                }
                factory_canvas::catalog_loader::CatalogJsonErrorKind::Syntax => {
                    "contains invalid JSON syntax"
                }
                factory_canvas::catalog_loader::CatalogJsonErrorKind::Schema => {
                    "does not match the expected schema"
                }
                factory_canvas::catalog_loader::CatalogJsonErrorKind::UnexpectedEndOfInput => {
                    "ends unexpectedly"
                }
            };
            format!(
                "The private {module} catalog module {description} at line {line}, column {column}."
            )
        }
        CatalogLoadError::UnsupportedSchemaVersion(_) => {
            "The private catalog schema version is not supported.".to_owned()
        }
        CatalogLoadError::InvalidDataVersion => {
            "The private catalog data_version is not valid SemVer.".to_owned()
        }
        CatalogLoadError::InvalidIdentifier {
            module,
            item_index,
            field,
        } => match item_index {
            Some(index) => format!(
                "The {field} field in private {module} item {} is not a valid identifier.",
                index + 1
            ),
            None => format!(
                "The {field} field in the private {module} catalog module is not a valid identifier."
            ),
        },
        CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field,
            ..
        } => format!(
            "The {field} field in {module} item {} has an invalid dimension.",
            item_index + 1
        ),
        CatalogLoadError::InvalidModulePath { module, kind } => {
            format!("The private {module} catalog module path {kind}.")
        }
        CatalogLoadError::DuplicateModulePath { first, second } => format!(
            "The private {first} and {second} catalog modules use the same path."
        ),
        CatalogLoadError::InvalidCatalog(_) => {
            "The private catalog failed integrity validation.".to_owned()
        }
    }
}

pub(super) fn notice_text(
    notice: &EditorNotice,
    current_base_name: &str,
    catalog: &Catalog,
) -> String {
    match notice {
        EditorNotice::SelectBlock => "Select a block to get started.".to_owned(),
        EditorNotice::ReadyToPlace { buildable_id } => format!(
            "Selected block: {}. Click the grid to place it.",
            buildable_for_id(buildable_id, catalog).display_name()
        ),
        EditorNotice::InstanceSelected { id, buildable_id } => format!(
            "Block #{} selected: {}.",
            id.value(),
            buildable_for_id(buildable_id, catalog).display_name()
        ),
        EditorNotice::InstancesSelected { count } => {
            format!("{count} blocks selected.")
        }
        EditorNotice::InstanceRemoved { id, buildable_id } => format!(
            "Block #{} removed: {}.",
            id.value(),
            buildable_for_id(buildable_id, catalog).display_name()
        ),
        EditorNotice::InstancesRemoved { count } => format!("{count} blocks removed."),
        EditorNotice::InstanceMoved { id, origin } => format!(
            "Block #{} moved to ({}, {}).",
            id.value(),
            origin.x,
            origin.y
        ),
        EditorNotice::InstancesMoved { count } => format!("{count} blocks moved."),
        EditorNotice::InstanceRotated { id, rotation } => {
            let degrees = match rotation {
                Rotation::Zero => 0,
                Rotation::Clockwise90 => 90,
                Rotation::Clockwise180 => 180,
                Rotation::Clockwise270 => 270,
            };
            format!("Block #{} rotated to {}°.", id.value(), degrees)
        }
        EditorNotice::InstancesRotated { count } => format!("{count} blocks rotated 90°."),
        EditorNotice::InstanceEditRejected(InstanceEditError::EntityNotFound { id }) => {
            format!("Block #{} no longer exists.", id.value())
        }
        EditorNotice::InstanceEditRejected(InstanceEditError::OutOfBounds { .. }) => {
            "The block does not fit at this position.".to_owned()
        }
        EditorNotice::InstanceEditRejected(InstanceEditError::Collision {
            conflicting_id, ..
        }) => {
            format!("Position occupied by block #{}.", conflicting_id.value())
        }
        EditorNotice::ProductionTargetChanged {
            id,
            product_id: Some(_),
        } => format!("Block #{} product updated.", id.value()),
        EditorNotice::ProductionTargetChanged {
            id,
            product_id: None,
        } => format!("Block #{} product cleared.", id.value()),
        EditorNotice::ProductionTargetRejected(ProductionTargetError::EntityNotFound { id }) => {
            format!("Block #{} no longer exists.", id.value())
        }
        EditorNotice::ProductionTargetRejected(ProductionTargetError::ProductNotFound {
            ..
        }) => "The selected product is not available in this catalog.".to_owned(),
        EditorNotice::ProductionTargetRejected(ProductionTargetError::UnsupportedProduct {
            ..
        }) => "The selected product is not supported by this construction.".to_owned(),
        EditorNotice::Placed {
            id,
            buildable_id,
            origin,
        } => format!(
            "Block #{} placed at ({}, {}): {}.",
            id.value(),
            origin.x,
            origin.y,
            buildable_for_id(buildable_id, catalog).display_name()
        ),
        EditorNotice::PlacementRejected(PlacementError::DuplicateEntityId { id }) => {
            format!("Internal ID #{} is already in use.", id.value())
        }
        EditorNotice::PlacementRejected(PlacementError::BuildableNotFound { .. }) => {
            "The selected construction is not available in this catalog.".to_owned()
        }
        EditorNotice::PlacementRejected(PlacementError::ProductNotFound { .. }) => {
            "The configured product is not available in this catalog.".to_owned()
        }
        EditorNotice::PlacementRejected(PlacementError::UnsupportedProduct { .. }) => {
            "The configured product is not supported by this construction.".to_owned()
        }
        EditorNotice::PlacementRejected(PlacementError::OutOfBounds { .. }) => {
            "The block does not fit at this position.".to_owned()
        }
        EditorNotice::PlacementRejected(PlacementError::Collision { conflicting_id, .. }) => {
            format!("Position occupied by block #{}.", conflicting_id.value())
        }
        EditorNotice::EntityIdsExhausted => "No IDs are available for new blocks.".to_owned(),
        EditorNotice::BaseChanged => format!("Base changed to {current_base_name}."),
        EditorNotice::DocumentSaved => "Factory saved.".to_owned(),
        EditorNotice::DocumentOpened(CatalogCompatibility::Exact) => "Factory opened.".to_owned(),
        EditorNotice::DocumentOpened(CatalogCompatibility::CatalogIdMismatch) => {
            "Factory opened with a catalog ID mismatch.".to_owned()
        }
        EditorNotice::DocumentOpened(CatalogCompatibility::DataVersionMismatch) => {
            "Factory opened with a catalog data version mismatch.".to_owned()
        }
        EditorNotice::DocumentOpened(CatalogCompatibility::CatalogAndDataVersionMismatch) => {
            "Factory opened with a catalog ID and data version mismatch.".to_owned()
        }
        EditorNotice::DocumentOpenFailed(error) => {
            format!("Factory could not be opened. {error}")
        }
        EditorNotice::DocumentSaveFailed(error) => {
            format!("Factory could not be saved. {error}")
        }
        EditorNotice::BlueprintSaved => "Blueprint saved.".to_owned(),
        EditorNotice::BlueprintSaveFailed(error) => {
            format!(
                "Blueprint could not be saved. {}",
                safe_blueprint_save_error_detail(error)
            )
        }
        EditorNotice::BlueprintArmedForInsertion => {
            "Blueprint ready to insert. Click the grid to place it.".to_owned()
        }
        EditorNotice::BlueprintInsertionUnavailable => {
            "This blueprint could not be read.".to_owned()
        }
        EditorNotice::BlueprintInserted { node_count } => {
            if *node_count == 1 {
                "Blueprint inserted: 1 block.".to_owned()
            } else {
                format!("Blueprint inserted: {node_count} blocks.")
            }
        }
        EditorNotice::BlueprintInsertionRejected(error) => {
            safe_blueprint_insertion_error_detail(error).to_owned()
        }
        EditorNotice::Undone => "Undone.".to_owned(),
        EditorNotice::Redone => "Redone.".to_owned(),
    }
}

/// Maps a `BlueprintInsertionError` to a fixed, generic, user-facing
/// message. Never echoes the failing node's index or the specific
/// buildable/product identifier — the same privacy discipline
/// `safe_blueprint_save_error_detail` already applies (FR-004/FR-005/
/// FR-006 only promise the player *that* the insertion was rejected and
/// *why in general terms*, not internal identifiers).
pub(super) fn safe_blueprint_insertion_error_detail(
    error: &BlueprintInsertionError,
) -> &'static str {
    match error {
        BlueprintInsertionError::CoordinateOverflow { .. } => {
            "the blueprint does not fit at this position."
        }
        BlueprintInsertionError::EntityIdsExhausted => "no IDs are available for new blocks.",
        BlueprintInsertionError::BuildableNotFound { .. } => {
            "the blueprint references a construction unavailable in this catalog."
        }
        BlueprintInsertionError::ProductNotFound { .. } => {
            "the blueprint references a product unavailable in this catalog."
        }
        BlueprintInsertionError::UnsupportedProduct { .. } => {
            "the blueprint references a product unsupported by one of its constructions."
        }
        BlueprintInsertionError::OutOfBounds { .. } => {
            "the blueprint does not fit at this position."
        }
        BlueprintInsertionError::Collision { .. } => {
            "the blueprint overlaps the existing layout at this position."
        }
    }
}

/// Maps a `BlueprintLibrarySaveError` to a fixed, generic, user-facing
/// message. Never formats or echoes the error's internals (`io::ErrorKind`,
/// the wrapped `BlueprintDocumentError`) — the same privacy discipline
/// `safe_catalog_load_detail` already applies to `CatalogLoadError`,
/// extended here to `BlueprintLibrarySaveError` for FR-011 (research.md
/// Decision 8).
pub(super) fn safe_blueprint_save_error_detail(error: &BlueprintLibrarySaveError) -> &'static str {
    match error {
        BlueprintLibrarySaveError::Io { .. } => {
            "the local blueprint storage location could not be written to."
        }
        BlueprintLibrarySaveError::Encoding(_) => "the blueprint could not be encoded.",
        BlueprintLibrarySaveError::IdCollisionExhausted => {
            "a unique identifier could not be generated. Please try again."
        }
    }
}

pub(super) fn notice_color(notice: &EditorNotice) -> Color32 {
    match notice {
        EditorNotice::PlacementRejected(_)
        | EditorNotice::InstanceEditRejected(_)
        | EditorNotice::ProductionTargetRejected(_)
        | EditorNotice::EntityIdsExhausted
        | EditorNotice::DocumentOpenFailed(_)
        | EditorNotice::DocumentSaveFailed(_)
        | EditorNotice::BlueprintSaveFailed(_)
        | EditorNotice::BlueprintInsertionUnavailable
        | EditorNotice::BlueprintInsertionRejected(_) => Color32::from_rgb(245, 132, 124),
        EditorNotice::DocumentOpened(CatalogCompatibility::Exact) => TEXT_MUTED,
        EditorNotice::DocumentOpened(_) => Color32::from_rgb(244, 190, 96),
        _ => TEXT_MUTED,
    }
}

/// A short, visible, non-blocking indication for a listed blueprint whose
/// stored catalog does not exactly match the currently active catalog
/// (FR-010), reusing the same three-variant wording already established by
/// `notice_text`'s `DocumentOpened(CatalogCompatibility::...)` arms for
/// factory documents, applied per-entry instead of as a one-shot notice.
/// `None` for an exact match — nothing is rendered in that case.
pub(super) fn catalog_compatibility_mismatch_text(
    compatibility: CatalogCompatibility,
) -> Option<&'static str> {
    match compatibility {
        CatalogCompatibility::Exact => None,
        CatalogCompatibility::CatalogIdMismatch => Some("Catalog ID mismatch"),
        CatalogCompatibility::DataVersionMismatch => Some("Catalog data version mismatch"),
        CatalogCompatibility::CatalogAndDataVersionMismatch => {
            Some("Catalog ID and data version mismatch")
        }
    }
}
