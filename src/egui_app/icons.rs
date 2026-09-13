use eframe::egui;
use factory_canvas::domain::catalog::{BuildableId, Catalog};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Raw icon files above this size are rejected before any decode is
/// attempted — see research.md Decision 3 (resource caps).
const MAX_ICON_FILE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IconPathError {
    Empty,
    NulByte,
    Rooted,
    WindowsPrefix,
    CurrentDirectory,
    ParentDirectory,
    OutsideRoot,
    Io,
    TooLarge,
    Decode,
}

impl fmt::Display for IconPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "is empty",
            Self::NulByte => "contains a NUL byte",
            Self::Rooted => "is rooted",
            Self::WindowsPrefix => "contains a Windows prefix or alternate stream separator",
            Self::CurrentDirectory => "contains a current-directory component",
            Self::ParentDirectory => "contains a parent-directory component",
            Self::OutsideRoot => "resolves outside the icon collection root",
            Self::Io => "could not be read",
            Self::TooLarge => "exceeds the maximum icon file size",
            Self::Decode => "could not be decoded as a supported image",
        })
    }
}

/// Validates and resolves a buildable's `icon` reference to an absolute
/// path inside `icons_root`, using the same normalize-then-canonicalize
/// algorithm `catalog_loader`'s `validate_module_path` +
/// `ensure_module_within_root` already use for catalog module paths
/// (research.md Decision 3). Never touches the filesystem beyond the
/// final `fs::canonicalize` calls needed to defeat a symlink/junction
/// escape.
pub(super) fn resolve_icon_path(
    icons_root: &Path,
    relative: &str,
) -> Result<PathBuf, IconPathError> {
    if relative.is_empty() {
        return Err(IconPathError::Empty);
    }
    if relative.contains('\0') {
        return Err(IconPathError::NulByte);
    }

    let normalized = relative.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(IconPathError::Rooted);
    }
    if normalized.contains(':') {
        return Err(IconPathError::WindowsPrefix);
    }
    for component in normalized.split('/') {
        match component {
            "" => return Err(IconPathError::Empty),
            "." => return Err(IconPathError::CurrentDirectory),
            ".." => return Err(IconPathError::ParentDirectory),
            _ => {}
        }
    }

    let canonical_root = fs::canonicalize(icons_root).map_err(|_| IconPathError::Io)?;
    let canonical_icon =
        fs::canonicalize(icons_root.join(&normalized)).map_err(|_| IconPathError::Io)?;

    if canonical_icon.starts_with(&canonical_root) {
        Ok(canonical_icon)
    } else {
        Err(IconPathError::OutsideRoot)
    }
}

/// Rejects a file whose raw byte length exceeds `MAX_ICON_FILE_BYTES`
/// before any decode is attempted (research.md Decision 3).
pub(super) fn check_icon_file_size(path: &Path) -> Result<(), IconPathError> {
    let metadata = fs::metadata(path).map_err(|_| IconPathError::Io)?;
    if metadata.len() > MAX_ICON_FILE_BYTES {
        Err(IconPathError::TooLarge)
    } else {
        Ok(())
    }
}

/// Decodes PNG bytes into an `egui::ColorImage`, rejecting any input
/// whose content is not actually PNG regardless of the file's
/// extension (research.md Decision 4 —
/// `image::load_from_memory_with_format` checks magic bytes, not the
/// filename).
pub(super) fn decode_png_rgba(bytes: &[u8]) -> Result<egui::ColorImage, IconPathError> {
    let dynamic_image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|_| IconPathError::Decode)?;
    let rgba = dynamic_image.to_rgba8();
    let size = [
        dynamic_image.width() as usize,
        dynamic_image.height() as usize,
    ];
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        rgba.as_raw(),
    ))
}

/// Every buildable's resolved icon texture, loaded once at startup.
/// Presentation-layer only — `resolve_icon_path`/`decode_png_rgba`
/// above are pure and I/O-bounded; `BuildableIcons::load` is the sole
/// caller that ties them to a live `egui::Context`
/// (research.md Decision 5).
#[derive(Default)]
pub(crate) struct BuildableIcons {
    textures: BTreeMap<BuildableId, egui::TextureHandle>,
    warnings: Vec<String>,
}

impl BuildableIcons {
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    pub(crate) fn texture(&self, id: &BuildableId) -> Option<&egui::TextureHandle> {
        self.textures.get(id)
    }

    /// The first sanitized icon-load warning, if any, shown non-blocking
    /// alongside `catalog_warning` (spec.md FR-004: icon failures are
    /// visible feedback, never a hard error). Only the first is
    /// surfaced to keep the header compact; every warning is still
    /// collected in `warnings` for `#[cfg(test)]` inspection.
    pub(crate) fn first_warning(&self) -> Option<&str> {
        self.warnings.first().map(String::as_str)
    }

    #[cfg(test)]
    pub(super) fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Loads every buildable's optional icon into a GPU texture. A
    /// per-icon failure at any step (unsafe path, missing file,
    /// oversized file, undecodable content) is isolated: it appends
    /// one sanitized warning and leaves that buildable without a
    /// texture, without aborting the loop or failing catalog loading
    /// (spec.md FR-004).
    pub(crate) fn load(ctx: &egui::Context, catalog: &Catalog, icons_root: &Path) -> Self {
        let mut textures = BTreeMap::new();
        let mut warnings = Vec::new();

        for buildable in catalog.buildables() {
            let Some(icon) = buildable.icon() else {
                continue;
            };
            if icon.is_empty() {
                continue;
            }

            match Self::load_one(ctx, icons_root, icon) {
                Ok(handle) => {
                    textures.insert(buildable.id().clone(), handle);
                }
                Err(error) => {
                    warnings.push(format!(
                        "Icon for buildable '{}' {error}.",
                        buildable.id().as_str()
                    ));
                }
            }
        }

        Self { textures, warnings }
    }

    fn load_one(
        ctx: &egui::Context,
        icons_root: &Path,
        relative: &str,
    ) -> Result<egui::TextureHandle, IconPathError> {
        let path = resolve_icon_path(icons_root, relative)?;
        check_icon_file_size(&path)?;
        let bytes = fs::read(&path).map_err(|_| IconPathError::Io)?;
        let color_image = decode_png_rgba(&bytes)?;
        Ok(ctx.load_texture(relative, color_image, egui::TextureOptions::LINEAR))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let unique = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "factory-canvas-icons-{label}-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test icons directory must be created");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[cfg(unix)]
    fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    fn create_file_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(target, link)
    }

    #[test]
    fn resolve_icon_path_rejects_structurally_unsafe_references() {
        let root = TestDirectory::new("structural");

        assert_eq!(
            resolve_icon_path(root.path(), ""),
            Err(IconPathError::Empty)
        );
        assert_eq!(
            resolve_icon_path(root.path(), "a\0b.png"),
            Err(IconPathError::NulByte)
        );
        assert_eq!(
            resolve_icon_path(root.path(), "/etc/passwd.png"),
            Err(IconPathError::Rooted)
        );
        assert_eq!(
            resolve_icon_path(root.path(), "C:\\Windows\\x.png"),
            Err(IconPathError::WindowsPrefix)
        );
        assert_eq!(
            resolve_icon_path(root.path(), "./x.png"),
            Err(IconPathError::CurrentDirectory)
        );
        assert_eq!(
            resolve_icon_path(root.path(), "../x.png"),
            Err(IconPathError::ParentDirectory)
        );
        assert_eq!(
            resolve_icon_path(root.path(), "sub/../../x.png"),
            Err(IconPathError::ParentDirectory)
        );
    }

    #[test]
    fn resolve_icon_path_accepts_a_real_file_inside_the_root() {
        let root = TestDirectory::new("accept");
        fs::write(root.path().join("x.png"), b"not a real png, just bytes")
            .expect("fixture file must be written");

        let resolved =
            resolve_icon_path(root.path(), "x.png").expect("a real file inside root must resolve");

        assert_eq!(
            resolved,
            fs::canonicalize(root.path().join("x.png")).unwrap()
        );
    }

    #[test]
    fn resolve_icon_path_rejects_a_symlink_escaping_the_root() {
        let root = TestDirectory::new("symlink-root");
        let outside = TestDirectory::new("symlink-target");
        let real_file = outside.path().join("real.png");
        fs::write(&real_file, b"outside bytes").expect("outside fixture must be written");
        let link = root.path().join("linked.png");

        if let Err(error) = create_file_symlink(&real_file, &link) {
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(1314)
            {
                eprintln!("skipping symlink-escape test: symlink privilege unavailable ({error})");
                return;
            }
            panic!("test symlink must be created: {error}");
        }

        assert_eq!(
            resolve_icon_path(root.path(), "linked.png"),
            Err(IconPathError::OutsideRoot)
        );
    }

    #[test]
    fn check_icon_file_size_rejects_files_above_the_cap() {
        let root = TestDirectory::new("size-cap");
        let oversized = root.path().join("big.png");
        let contents = vec![0_u8; (MAX_ICON_FILE_BYTES + 1) as usize];
        fs::write(&oversized, &contents).expect("oversized fixture must be written");

        assert_eq!(
            check_icon_file_size(&oversized),
            Err(IconPathError::TooLarge)
        );
    }

    #[test]
    fn check_icon_file_size_accepts_files_at_or_below_the_cap() {
        let root = TestDirectory::new("size-ok");
        let ok_file = root.path().join("ok.png");
        fs::write(&ok_file, vec![0_u8; 16]).expect("small fixture must be written");

        assert_eq!(check_icon_file_size(&ok_file), Ok(()));
    }

    /// A minimal valid 1x1 opaque red PNG, used to exercise the real
    /// decode path without depending on an external test asset.
    /// Generated once and embedded as a byte literal — standard PNG
    /// signature + IHDR + IDAT + IEND, 1x1 RGB, no filter.
    const ONE_PIXEL_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0xF7, 0x03, 0x41, 0x43, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn decode_png_rgba_rejects_non_png_bytes() {
        let jpeg_magic = [0xFF_u8, 0xD8, 0xFF, 0xE0, 0x00, 0x10];

        assert_eq!(decode_png_rgba(&jpeg_magic), Err(IconPathError::Decode));
    }

    #[test]
    fn decode_png_rgba_decodes_a_real_png_to_matching_dimensions() {
        let image = decode_png_rgba(ONE_PIXEL_PNG).expect("the embedded 1x1 PNG must decode");

        assert_eq!(image.size, [1, 1]);
    }

    fn synthetic_catalog_with_icon(icon: Option<&str>) -> Catalog {
        synthetic_catalog_with_buildable_icons(&[("icon_machine", icon)])
    }

    fn synthetic_catalog_with_buildable_icons(entries: &[(&str, Option<&str>)]) -> Catalog {
        use factory_canvas::domain::catalog::{
            BaseDefinition, BaseId, CatalogId, CatalogMetadata, CategoryId, RegionDefinition,
            RegionId,
        };
        use factory_canvas::domain::catalog::{BuildableDefinition, ProductDefinition};
        use factory_canvas::domain::geometry::GridSize;
        use semver::Version;

        let region_id = RegionId::new("test_region").expect("valid region ID");
        let base_id = BaseId::new("test_base").expect("valid base ID");
        let buildables = entries
            .iter()
            .map(|(id, icon)| {
                BuildableDefinition::new(
                    BuildableId::new(*id).expect("valid buildable ID"),
                    *id,
                    CategoryId::new("test_category").expect("valid category ID"),
                    "T",
                    GridSize::new(1, 1).expect("positive footprint"),
                    Vec::<ProductDefinition>::new()
                        .iter()
                        .map(|p: &ProductDefinition| p.id().clone())
                        .collect(),
                    *icon,
                )
            })
            .collect();
        Catalog::new(
            CatalogMetadata::new(
                CatalogId::new("test_catalog").expect("valid catalog ID"),
                Version::new(1, 0, 0),
                "Test Catalog",
            ),
            base_id.clone(),
            vec![RegionDefinition::new(region_id.clone(), "Test Region")],
            vec![BaseDefinition::new(
                base_id,
                "Test Base",
                region_id,
                GridSize::new(10, 10).expect("positive bounds"),
            )],
            buildables,
            Vec::new(),
        )
        .expect("synthetic test catalog must be valid")
    }

    #[test]
    fn load_populates_a_texture_for_a_valid_icon_and_no_warning() {
        let root = TestDirectory::new("load-valid");
        fs::write(root.path().join("icon_machine.png"), ONE_PIXEL_PNG)
            .expect("PNG fixture must be written");
        let catalog = synthetic_catalog_with_icon(Some("icon_machine.png"));
        let context = egui::Context::default();

        let icons = BuildableIcons::load(&context, &catalog, root.path());

        let id = BuildableId::new("icon_machine").expect("valid buildable ID");
        assert!(icons.texture(&id).is_some());
        assert_eq!(icons.first_warning(), None);
        assert!(icons.warnings().is_empty());
    }

    #[test]
    fn load_falls_back_to_no_texture_and_one_warning_when_the_file_is_missing() {
        let root = TestDirectory::new("load-missing");
        let catalog = synthetic_catalog_with_icon(Some("does_not_exist.png"));
        let context = egui::Context::default();

        let icons = BuildableIcons::load(&context, &catalog, root.path());

        let id = BuildableId::new("icon_machine").expect("valid buildable ID");
        assert!(icons.texture(&id).is_none());
        assert_eq!(icons.warnings().len(), 1);
        assert!(icons.first_warning().unwrap().contains("icon_machine"));
    }

    #[test]
    fn load_is_silent_when_no_buildable_declares_an_icon() {
        let root = TestDirectory::new("load-none");
        let catalog = synthetic_catalog_with_icon(None);
        let context = egui::Context::default();

        let icons = BuildableIcons::load(&context, &catalog, root.path());

        assert!(icons.warnings().is_empty());
        assert_eq!(icons.first_warning(), None);
    }

    #[test]
    fn load_isolates_one_buildables_failure_from_anothers_success() {
        // T038: a catalog with one valid and one missing icon in the
        // SAME load() call — confirms failure isolation across
        // buildables, not merely that a single-buildable load can
        // independently succeed or fail (the two existing tests above
        // each only exercise one buildable at a time).
        let root = TestDirectory::new("load-mixed");
        fs::write(root.path().join("valid.png"), ONE_PIXEL_PNG)
            .expect("PNG fixture must be written");
        let catalog = synthetic_catalog_with_buildable_icons(&[
            ("valid_machine", Some("valid.png")),
            ("broken_machine", Some("missing.png")),
        ]);
        let context = egui::Context::default();

        let icons = BuildableIcons::load(&context, &catalog, root.path());

        let valid_id = BuildableId::new("valid_machine").expect("valid buildable ID");
        let broken_id = BuildableId::new("broken_machine").expect("valid buildable ID");
        assert!(
            icons.texture(&valid_id).is_some(),
            "the valid buildable's icon must still load despite the other's failure"
        );
        assert!(icons.texture(&broken_id).is_none());
        assert_eq!(icons.warnings().len(), 1);
        assert!(icons.first_warning().unwrap().contains("broken_machine"));
    }

    #[test]
    fn load_treats_blank_icon_identically_to_absent_and_warns_only_for_unusable_references() {
        // T040: omitted (None), explicit blank (""), and an explicit
        // unusable reference must be distinguishable by warning count
        // — None/"" are silent (0 warnings), an unusable reference
        // warns exactly once.
        let root = TestDirectory::new("load-blank-vs-unusable");
        let catalog = synthetic_catalog_with_buildable_icons(&[
            ("omitted_machine", None),
            ("blank_machine", Some("")),
            ("unusable_machine", Some("still_missing.png")),
        ]);
        let context = egui::Context::default();

        let icons = BuildableIcons::load(&context, &catalog, root.path());

        assert_eq!(icons.warnings().len(), 1);
        assert!(icons.first_warning().unwrap().contains("unusable_machine"));
        for id in ["omitted_machine", "blank_machine", "unusable_machine"] {
            assert!(icons
                .texture(&BuildableId::new(id).expect("valid buildable ID"))
                .is_none());
        }
    }

    #[test]
    fn load_rejects_a_path_traversal_reference_end_to_end() {
        // T042: confirms resolve_icon_path's OutsideRoot rejection
        // (already unit-tested directly in
        // resolve_icon_path_rejects_structurally_unsafe_references) is
        // actually wired into the full BuildableIcons::load path, not
        // merely correct in isolation. Uses a real file that exists
        // one level above icons_root so a bug that silently allowed
        // the traversal would make this test fail by unexpectedly
        // succeeding, not merely by a missing-file error looking the
        // same as a rejected one.
        let parent = TestDirectory::new("traversal-parent");
        let icons_root = parent.path().join("icons");
        fs::create_dir(&icons_root).expect("icons_root must be created");
        fs::write(parent.path().join("outside.png"), ONE_PIXEL_PNG)
            .expect("outside fixture must be written");
        let catalog = synthetic_catalog_with_icon(Some("../outside.png"));
        let context = egui::Context::default();

        let icons = BuildableIcons::load(&context, &catalog, &icons_root);

        let id = BuildableId::new("icon_machine").expect("valid buildable ID");
        assert!(icons.texture(&id).is_none());
        assert_eq!(icons.warnings().len(), 1);
    }
}
