use super::directory_source::{CatalogSource, CatalogSourceError};
use super::dto::DimensionsDto;
use super::errors::{CatalogJsonErrorKind, CatalogLoadError, CatalogPathErrorKind};
use super::CatalogModule;
use crate::domain::catalog::IdentifierError;
use crate::domain::geometry::{GridSize, GridSizeError};
use serde::de::DeserializeOwned;

pub(super) fn read_module(
    source: &impl CatalogSource,
    module: CatalogModule,
    path: &str,
) -> Result<String, CatalogLoadError> {
    source.read_module(path).map_err(|error| match error {
        CatalogSourceError::Io(kind) => CatalogLoadError::ModuleRead { module, kind },
        CatalogSourceError::OutsideRoot => CatalogLoadError::ModuleOutsideRoot { module },
    })
}

pub(super) fn validate_module_path(
    raw_path: &str,
    module: CatalogModule,
) -> Result<String, CatalogLoadError> {
    let invalid = |kind| CatalogLoadError::InvalidModulePath { module, kind };

    if raw_path.trim().is_empty() {
        return Err(invalid(CatalogPathErrorKind::Empty));
    }
    if raw_path.contains('\0') {
        return Err(invalid(CatalogPathErrorKind::NulByte));
    }

    let normalized = raw_path.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(invalid(CatalogPathErrorKind::Rooted));
    }
    if normalized.contains(':') {
        return Err(invalid(CatalogPathErrorKind::WindowsPrefix));
    }

    for component in normalized.split('/') {
        match component {
            "" => return Err(invalid(CatalogPathErrorKind::EmptyComponent)),
            "." => return Err(invalid(CatalogPathErrorKind::CurrentDirectory)),
            ".." => return Err(invalid(CatalogPathErrorKind::ParentDirectory)),
            _ => {}
        }
    }

    Ok(normalized)
}

pub(super) fn validate_unique_module_paths(
    paths: [(CatalogModule, &str); 4],
) -> Result<(), CatalogLoadError> {
    for (index, (first_module, first_path)) in paths.iter().enumerate() {
        for (second_module, second_path) in &paths[index + 1..] {
            if first_path == second_path {
                return Err(CatalogLoadError::DuplicateModulePath {
                    first: *first_module,
                    second: *second_module,
                });
            }
        }
    }

    Ok(())
}

pub(super) fn parse_json<T: DeserializeOwned>(
    text: &str,
    module: CatalogModule,
) -> Result<T, CatalogLoadError> {
    serde_json::from_str(text).map_err(|error| CatalogLoadError::InvalidJson {
        module,
        kind: match error.classify() {
            serde_json::error::Category::Io => CatalogJsonErrorKind::Io,
            serde_json::error::Category::Syntax => CatalogJsonErrorKind::Syntax,
            serde_json::error::Category::Data => CatalogJsonErrorKind::Schema,
            serde_json::error::Category::Eof => CatalogJsonErrorKind::UnexpectedEndOfInput,
        },
        line: error.line(),
        column: error.column(),
    })
}

pub(super) fn parse_identifier<T>(
    value: String,
    module: CatalogModule,
    item_index: Option<usize>,
    field: &'static str,
    constructor: impl FnOnce(String) -> Result<T, IdentifierError>,
) -> Result<T, CatalogLoadError> {
    constructor(value).map_err(|_| CatalogLoadError::InvalidIdentifier {
        module,
        item_index,
        field,
    })
}

pub(super) fn parse_dimensions(
    dimensions: DimensionsDto,
    module: CatalogModule,
    item_index: usize,
) -> Result<GridSize, CatalogLoadError> {
    let width =
        u16::try_from(dimensions.width).map_err(|_| CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field: "width",
            value: dimensions.width,
        })?;
    let height =
        u16::try_from(dimensions.height).map_err(|_| CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field: "height",
            value: dimensions.height,
        })?;

    GridSize::new(width, height).map_err(|error| {
        let (field, value) = match error {
            GridSizeError::ZeroWidth => ("width", dimensions.width),
            GridSizeError::ZeroHeight => ("height", dimensions.height),
        };
        CatalogLoadError::InvalidDimension {
            module,
            item_index,
            field,
            value,
        }
    })
}
