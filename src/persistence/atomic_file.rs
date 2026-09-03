use std::io::{self, Write};
use std::path::Path;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtomicWriteStage {
    CreateTemporary,
    WriteTemporary,
    FlushTemporary,
    SyncTemporary,
    ReplaceTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AtomicWriteError {
    stage: AtomicWriteStage,
    kind: io::ErrorKind,
}

impl AtomicWriteError {
    pub(crate) const fn stage(self) -> AtomicWriteStage {
        self.stage
    }

    pub(crate) const fn kind(self) -> io::ErrorKind {
        self.kind
    }
}

pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), AtomicWriteError> {
    write_atomically_with(path, bytes, |temporary, target| {
        temporary
            .persist(target)
            .map(|_| ())
            .map_err(|error| AtomicWriteError {
                stage: AtomicWriteStage::ReplaceTarget,
                kind: error.error.kind(),
            })
    })
}

fn write_atomically_with(
    path: &Path,
    bytes: &[u8],
    replace: impl FnOnce(NamedTempFile, &Path) -> Result<(), AtomicWriteError>,
) -> Result<(), AtomicWriteError> {
    let parent = path.parent().ok_or(AtomicWriteError {
        stage: AtomicWriteStage::CreateTemporary,
        kind: io::ErrorKind::InvalidInput,
    })?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| AtomicWriteError {
        stage: AtomicWriteStage::CreateTemporary,
        kind: error.kind(),
    })?;
    temporary
        .write_all(bytes)
        .map_err(|error| AtomicWriteError {
            stage: AtomicWriteStage::WriteTemporary,
            kind: error.kind(),
        })?;
    temporary.flush().map_err(|error| AtomicWriteError {
        stage: AtomicWriteStage::FlushTemporary,
        kind: error.kind(),
    })?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| AtomicWriteError {
            stage: AtomicWriteStage::SyncTemporary,
            kind: error.kind(),
        })?;
    replace(temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn failed_atomic_replace_preserves_previous_target_bytes() {
        let directory = tempfile::tempdir().expect("test directory must be created");
        let target = directory.path().join("factory.json");
        let previous = b"previous complete document\n";
        fs::write(&target, previous).expect("previous target must be created");

        let result = write_atomically_with(&target, b"replacement\n", |_, _| {
            Err(AtomicWriteError {
                stage: AtomicWriteStage::ReplaceTarget,
                kind: io::ErrorKind::Other,
            })
        });

        assert_eq!(
            result,
            Err(AtomicWriteError {
                stage: AtomicWriteStage::ReplaceTarget,
                kind: io::ErrorKind::Other,
            })
        );
        assert_eq!(
            fs::read(&target).expect("previous target must remain readable"),
            previous
        );
        let entries = fs::read_dir(directory.path())
            .expect("test directory must be readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("test directory entries must be readable");
        assert_eq!(entries.len(), 1, "temporary file must be cleaned up");
        assert_eq!(entries[0].path(), target);
    }
}
