use std::fs;
use std::path::{Path, PathBuf};

pub(super) trait CatalogSource {
    fn read_manifest(&self) -> Result<String, std::io::ErrorKind>;
    fn read_module(&self, path: &str) -> Result<String, CatalogSourceError>;
}

pub(super) enum CatalogSourceError {
    Io(std::io::ErrorKind),
    OutsideRoot,
}

pub(super) struct DirectorySource {
    root: PathBuf,
}

impl DirectorySource {
    pub(super) fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_owned(),
        }
    }
}

impl CatalogSource for DirectorySource {
    fn read_manifest(&self) -> Result<String, std::io::ErrorKind> {
        fs::read_to_string(self.root.join("manifest.json")).map_err(|error| error.kind())
    }

    fn read_module(&self, path: &str) -> Result<String, CatalogSourceError> {
        let canonical_root =
            fs::canonicalize(&self.root).map_err(|error| CatalogSourceError::Io(error.kind()))?;
        let canonical_module = fs::canonicalize(self.root.join(path))
            .map_err(|error| CatalogSourceError::Io(error.kind()))?;

        ensure_module_within_root(&canonical_root, &canonical_module)?;

        fs::read_to_string(canonical_module).map_err(|error| CatalogSourceError::Io(error.kind()))
    }
}

pub(super) fn ensure_module_within_root(
    canonical_root: &Path,
    canonical_module: &Path,
) -> Result<(), CatalogSourceError> {
    if canonical_module.starts_with(canonical_root) {
        Ok(())
    } else {
        Err(CatalogSourceError::OutsideRoot)
    }
}

pub(super) struct EmbeddedPublicSource;

impl CatalogSource for EmbeddedPublicSource {
    fn read_manifest(&self) -> Result<String, std::io::ErrorKind> {
        Ok(include_str!("../../catalog/public/manifest.json").to_owned())
    }

    fn read_module(&self, path: &str) -> Result<String, CatalogSourceError> {
        match path {
            "regions.json" => Ok(include_str!("../../catalog/public/regions.json").to_owned()),
            "bases.json" => Ok(include_str!("../../catalog/public/bases.json").to_owned()),
            "buildables.json" => {
                Ok(include_str!("../../catalog/public/buildables.json").to_owned())
            }
            "products.json" => Ok(include_str!("../../catalog/public/products.json").to_owned()),
            _ => Err(CatalogSourceError::Io(std::io::ErrorKind::NotFound)),
        }
    }
}
