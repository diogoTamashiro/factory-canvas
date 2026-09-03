use super::catalog::{Catalog, CatalogId};
use semver::Version;
use std::fmt;
use std::sync::Arc;
use time::{OffsetDateTime, UtcOffset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentMetadataError {
    BlankName,
    UpdatedBeforeCreated,
}

impl fmt::Display for DocumentMetadataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankName => formatter.write_str("document name must not be blank"),
            Self::UpdatedBeforeCreated => {
                formatter.write_str("document update time must not precede its creation time")
            }
        }
    }
}

impl std::error::Error for DocumentMetadataError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    name: Arc<str>,
    description: Option<Arc<str>>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl DocumentMetadata {
    pub fn new(
        name: &str,
        description: Option<&str>,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Result<Self, DocumentMetadataError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(DocumentMetadataError::BlankName);
        }
        if updated_at < created_at {
            return Err(DocumentMetadataError::UpdatedBeforeCreated);
        }

        Ok(Self {
            name: Arc::from(name),
            description: description
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(Arc::from),
            created_at: created_at.to_offset(UtcOffset::UTC),
            updated_at: updated_at.to_offset(UtcOffset::UTC),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    pub const fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProvenance {
    catalog_id: CatalogId,
    data_version: Version,
}

impl CatalogProvenance {
    pub fn from_catalog(catalog: &Catalog) -> Self {
        Self {
            catalog_id: catalog.metadata().catalog_id().clone(),
            data_version: catalog.metadata().data_version().clone(),
        }
    }

    pub fn catalog_id(&self) -> &CatalogId {
        &self.catalog_id
    }

    pub fn data_version(&self) -> &Version {
        &self.data_version
    }
}
