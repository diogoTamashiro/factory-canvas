use super::CatalogModule;
use crate::domain::catalog::CatalogValidationError;
use std::fmt;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogJsonErrorKind {
    Io,
    Syntax,
    Schema,
    UnexpectedEndOfInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogPathErrorKind {
    Empty,
    Rooted,
    WindowsPrefix,
    CurrentDirectory,
    ParentDirectory,
    EmptyComponent,
    NulByte,
}

impl fmt::Display for CatalogPathErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "is empty",
            Self::Rooted => "is rooted",
            Self::WindowsPrefix => "contains a Windows prefix or alternate stream separator",
            Self::CurrentDirectory => "contains a current-directory component",
            Self::ParentDirectory => "contains a parent-directory component",
            Self::EmptyComponent => "contains an empty component",
            Self::NulByte => "contains a NUL byte",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogLoadError {
    ManifestRead(io::ErrorKind),
    ModuleRead {
        module: CatalogModule,
        kind: io::ErrorKind,
    },
    ModuleOutsideRoot {
        module: CatalogModule,
    },
    InvalidJson {
        module: CatalogModule,
        kind: CatalogJsonErrorKind,
        line: usize,
        column: usize,
    },
    UnsupportedSchemaVersion(u64),
    InvalidDataVersion,
    InvalidIdentifier {
        module: CatalogModule,
        item_index: Option<usize>,
        field: &'static str,
    },
    InvalidDimension {
        module: CatalogModule,
        item_index: usize,
        field: &'static str,
        value: u64,
    },
    InvalidModulePath {
        module: CatalogModule,
        kind: CatalogPathErrorKind,
    },
    DuplicateModulePath {
        first: CatalogModule,
        second: CatalogModule,
    },
    InvalidCatalog(CatalogValidationError),
}

impl CatalogLoadError {
    pub fn is_manifest_not_found(&self) -> bool {
        matches!(self, Self::ManifestRead(io::ErrorKind::NotFound))
    }
}

impl fmt::Display for CatalogLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestRead(kind) => {
                write!(formatter, "The catalog manifest could not be read ({kind}).")
            }
            Self::ModuleRead { module, kind } => {
                write!(
                    formatter,
                    "The {module} catalog module could not be read ({kind:?})."
                )
            }
            Self::ModuleOutsideRoot { module } => write!(
                formatter,
                "The {module} catalog module resolves outside the package root."
            ),
            Self::InvalidJson {
                module,
                kind,
                line,
                column,
            } => {
                let description = match kind {
                    CatalogJsonErrorKind::Io => "could not be decoded",
                    CatalogJsonErrorKind::Syntax => "contains invalid JSON syntax",
                    CatalogJsonErrorKind::Schema => "does not match the expected schema",
                    CatalogJsonErrorKind::UnexpectedEndOfInput => "ends unexpectedly",
                };
                write!(
                    formatter,
                    "The {module} catalog module {description} at line {line}, column {column}."
                )
            }
            Self::UnsupportedSchemaVersion(version) => write!(
                formatter,
                "The catalog schema version {version} is not supported."
            ),
            Self::InvalidDataVersion => {
                formatter.write_str("The catalog data_version is not valid SemVer.")
            }
            Self::InvalidIdentifier {
                module,
                item_index,
                field,
            } => match item_index {
                Some(index) => write!(
                    formatter,
                    "The {field} field in {module} item {} is not a valid ASCII snake_case identifier.",
                    index + 1
                ),
                None => write!(
                    formatter,
                    "The {field} field in the {module} is not a valid ASCII snake_case identifier."
                ),
            },
            Self::InvalidDimension {
                module,
                item_index,
                field,
                value,
            } => write!(
                formatter,
                "The {field} field in {module} item {} has invalid dimension {value}.",
                item_index + 1
            ),
            Self::InvalidModulePath { module, kind } => {
                write!(formatter, "The {module} catalog module path {kind}.")
            }
            Self::DuplicateModulePath { first, second } => write!(
                formatter,
                "The {first} and {second} catalog modules use the same path."
            ),
            Self::InvalidCatalog(error) => {
                write!(formatter, "The catalog failed integrity validation: {error}.")
            }
        }
    }
}

impl std::error::Error for CatalogLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidCatalog(error) => Some(error),
            _ => None,
        }
    }
}
