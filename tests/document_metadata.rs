use factory_canvas::domain::document::{DocumentMetadata, DocumentMetadataError};
use time::{OffsetDateTime, UtcOffset};

#[test]
fn document_metadata_normalizes_timestamps_to_utc() {
    let offset = UtcOffset::from_hms(5, 30, 0).unwrap();
    let created_at = OffsetDateTime::from_unix_timestamp(1_700_000_000)
        .unwrap()
        .to_offset(offset);
    let updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_123)
        .unwrap()
        .to_offset(offset);

    let metadata = DocumentMetadata::new("Factory", None, created_at, updated_at).unwrap();

    assert_eq!(metadata.created_at().offset(), UtcOffset::UTC);
    assert_eq!(metadata.updated_at().offset(), UtcOffset::UTC);
}

#[test]
fn document_metadata_rejects_a_blank_name() {
    let timestamp = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

    assert_eq!(
        DocumentMetadata::new("  \t\n", None, timestamp, timestamp),
        Err(DocumentMetadataError::BlankName)
    );
}

#[test]
fn document_metadata_rejects_an_update_before_creation() {
    let created_at = OffsetDateTime::from_unix_timestamp(1_700_000_100).unwrap();
    let updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

    assert_eq!(
        DocumentMetadata::new("Factory", None, created_at, updated_at),
        Err(DocumentMetadataError::UpdatedBeforeCreated)
    );
}

#[test]
fn document_metadata_trims_text_and_normalizes_blank_description() {
    let timestamp = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let metadata =
        DocumentMetadata::new("  Factory  ", Some(" \t "), timestamp, timestamp).unwrap();

    assert_eq!(metadata.name(), "Factory");
    assert_eq!(metadata.description(), None);
}
