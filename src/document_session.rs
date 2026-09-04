use factory_canvas::domain::document::DocumentMetadata;
use factory_canvas::domain::layout::FactoryLayout;
use factory_canvas::persistence::factory_document::{
    save_factory_document, CatalogCompatibility, FactoryDocumentError,
};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

#[derive(Debug)]
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "file commands are wired in the next atomic commit"
    )
)]
pub(crate) struct DocumentSession {
    path: Option<PathBuf>,
    metadata: DocumentMetadata,
    compatibility: CatalogCompatibility,
    dirty: bool,
}

impl Default for DocumentSession {
    fn default() -> Self {
        Self::untitled_at(OffsetDateTime::now_utc())
    }
}

#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "file commands are wired in the next atomic commit"
    )
)]
impl DocumentSession {
    pub(crate) fn untitled_at(created_at: OffsetDateTime) -> Self {
        let metadata = DocumentMetadata::new("Untitled Factory", None, created_at, created_at)
            .expect("untitled document metadata is statically valid");
        Self {
            path: None,
            metadata,
            compatibility: CatalogCompatibility::Exact,
            dirty: false,
        }
    }

    pub(crate) fn loaded(
        path: &Path,
        metadata: DocumentMetadata,
        compatibility: CatalogCompatibility,
    ) -> Self {
        Self {
            path: Some(path.to_path_buf()),
            metadata,
            compatibility,
            dirty: false,
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) fn metadata(&self) -> &DocumentMetadata {
        &self.metadata
    }

    pub(crate) const fn compatibility(&self) -> CatalogCompatibility {
        self.compatibility
    }

    pub(crate) const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(crate) fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn save_to(
        &mut self,
        path: &Path,
        layout: &FactoryLayout,
        next_entity_id: Option<u64>,
        saved_at: OffsetDateTime,
    ) -> Result<(), FactoryDocumentError> {
        let updated_at = saved_at.max(self.metadata.updated_at());
        let metadata = DocumentMetadata::new(
            self.metadata.name(),
            self.metadata.description(),
            self.metadata.created_at(),
            updated_at,
        )
        .map_err(FactoryDocumentError::InvalidMetadata)?;
        save_factory_document(path, layout, next_entity_id, &metadata)?;

        self.path = Some(path.to_path_buf());
        self.metadata = metadata;
        self.compatibility = CatalogCompatibility::Exact;
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use factory_canvas::persistence::factory_document::CatalogCompatibility;
    use time::OffsetDateTime;

    #[test]
    fn untitled_session_is_clean_unassociated_and_exact() {
        let session = DocumentSession::untitled_at(OffsetDateTime::UNIX_EPOCH);

        assert!(!session.is_dirty());
        assert_eq!(session.path(), None);
        assert_eq!(session.metadata().name(), "Untitled Factory");
        assert_eq!(session.metadata().description(), None);
        assert_eq!(session.metadata().created_at(), OffsetDateTime::UNIX_EPOCH);
        assert_eq!(session.metadata().updated_at(), OffsetDateTime::UNIX_EPOCH);
        assert_eq!(session.compatibility(), CatalogCompatibility::Exact);
    }
}
