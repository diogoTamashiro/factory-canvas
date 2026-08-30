use super::geometry::GridSize;
use semver::Version;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentifierError;

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "identifier must be non-empty ASCII snake_case beginning with a lowercase letter",
        )
    }
}

impl std::error::Error for IdentifierError {}

macro_rules! define_identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Arc<str>);

        impl $name {
            pub fn new(value: impl Into<Arc<str>>) -> Result<Self, IdentifierError> {
                let value = value.into();
                if !is_valid_identifier(&value) {
                    return Err(IdentifierError);
                }

                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

define_identifier!(CatalogId);
define_identifier!(RegionId);
define_identifier!(BaseId);
define_identifier!(BuildableId);
define_identifier!(ProductId);
define_identifier!(CategoryId);

fn is_valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase()) {
        return false;
    }

    let mut previous_was_underscore = false;
    for byte in bytes {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => previous_was_underscore = false,
            b'_' if !previous_was_underscore => previous_was_underscore = true,
            _ => return false,
        }
    }

    !previous_was_underscore
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogMetadata {
    catalog_id: CatalogId,
    data_version: Version,
    display_name: Arc<str>,
}

impl CatalogMetadata {
    pub fn new(
        catalog_id: CatalogId,
        data_version: Version,
        display_name: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            catalog_id,
            data_version,
            display_name: display_name.into(),
        }
    }

    pub fn catalog_id(&self) -> &CatalogId {
        &self.catalog_id
    }

    pub fn data_version(&self) -> &Version {
        &self.data_version
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionDefinition {
    id: RegionId,
    display_name: Arc<str>,
}

impl RegionDefinition {
    pub fn new(id: RegionId, display_name: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
        }
    }

    pub fn id(&self) -> &RegionId {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseDefinition {
    id: BaseId,
    display_name: Arc<str>,
    region_id: RegionId,
    bounds: GridSize,
}

impl BaseDefinition {
    pub fn new(
        id: BaseId,
        display_name: impl Into<Arc<str>>,
        region_id: RegionId,
        bounds: GridSize,
    ) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            region_id,
            bounds,
        }
    }

    pub fn id(&self) -> &BaseId {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn region_id(&self) -> &RegionId {
        &self.region_id
    }

    pub fn bounds(&self) -> GridSize {
        self.bounds
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildableDefinition {
    id: BuildableId,
    display_name: Arc<str>,
    category_id: CategoryId,
    symbol: Arc<str>,
    footprint: GridSize,
    production_targets: Vec<ProductId>,
}

impl BuildableDefinition {
    pub fn new(
        id: BuildableId,
        display_name: impl Into<Arc<str>>,
        category_id: CategoryId,
        symbol: impl Into<Arc<str>>,
        footprint: GridSize,
        production_targets: Vec<ProductId>,
    ) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            category_id,
            symbol: symbol.into(),
            footprint,
            production_targets,
        }
    }

    pub fn id(&self) -> &BuildableId {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category_id(&self) -> &CategoryId {
        &self.category_id
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn footprint(&self) -> GridSize {
        self.footprint
    }

    pub fn production_targets(&self) -> &[ProductId] {
        &self.production_targets
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDefinition {
    id: ProductId,
    display_name: Arc<str>,
}

impl ProductDefinition {
    pub fn new(id: ProductId, display_name: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
        }
    }

    pub fn id(&self) -> &ProductId {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogValidationError {
    EmptyCatalogDisplayName,
    EmptyRegionDisplayName(RegionId),
    EmptyBaseDisplayName(BaseId),
    EmptyBuildableDisplayName(BuildableId),
    InvalidBuildableSymbol(BuildableId),
    EmptyProductDisplayName(ProductId),
    DuplicateRegionId(RegionId),
    DuplicateBaseId(BaseId),
    DuplicateBuildableId(BuildableId),
    DuplicateProductId(ProductId),
    MissingDefaultBase(BaseId),
    MissingRegion {
        base_id: BaseId,
        region_id: RegionId,
    },
    MissingProductionTarget {
        buildable_id: BuildableId,
        product_id: ProductId,
    },
    DuplicateProductionTarget {
        buildable_id: BuildableId,
        product_id: ProductId,
    },
}

impl fmt::Display for CatalogValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalogDisplayName => {
                formatter.write_str("catalog display name must not be blank")
            }
            Self::EmptyRegionDisplayName(id) => write!(
                formatter,
                "region '{}' display name must not be blank",
                id.as_str()
            ),
            Self::EmptyBaseDisplayName(id) => write!(
                formatter,
                "base '{}' display name must not be blank",
                id.as_str()
            ),
            Self::EmptyBuildableDisplayName(id) => write!(
                formatter,
                "buildable '{}' display name must not be blank",
                id.as_str()
            ),
            Self::InvalidBuildableSymbol(id) => write!(
                formatter,
                "buildable '{}' symbol must contain one to four trimmed characters",
                id.as_str()
            ),
            Self::EmptyProductDisplayName(id) => write!(
                formatter,
                "product '{}' display name must not be blank",
                id.as_str()
            ),
            Self::DuplicateRegionId(id) => {
                write!(formatter, "region ID '{}' is duplicated", id.as_str())
            }
            Self::DuplicateBaseId(id) => {
                write!(formatter, "base ID '{}' is duplicated", id.as_str())
            }
            Self::DuplicateBuildableId(id) => {
                write!(formatter, "buildable ID '{}' is duplicated", id.as_str())
            }
            Self::DuplicateProductId(id) => {
                write!(formatter, "product ID '{}' is duplicated", id.as_str())
            }
            Self::MissingDefaultBase(id) => {
                write!(formatter, "default base '{}' does not exist", id.as_str())
            }
            Self::MissingRegion { base_id, region_id } => write!(
                formatter,
                "base '{}' references missing region '{}'",
                base_id.as_str(),
                region_id.as_str()
            ),
            Self::MissingProductionTarget {
                buildable_id,
                product_id,
            } => write!(
                formatter,
                "buildable '{}' references missing product '{}'",
                buildable_id.as_str(),
                product_id.as_str()
            ),
            Self::DuplicateProductionTarget {
                buildable_id,
                product_id,
            } => write!(
                formatter,
                "buildable '{}' repeats production target '{}'",
                buildable_id.as_str(),
                product_id.as_str()
            ),
        }
    }
}

impl std::error::Error for CatalogValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    inner: Arc<CatalogData>,
}

#[derive(Debug, PartialEq, Eq)]
struct CatalogData {
    metadata: CatalogMetadata,
    default_base_id: BaseId,
    regions: Vec<RegionDefinition>,
    region_index: BTreeMap<RegionId, usize>,
    bases: Vec<BaseDefinition>,
    base_index: BTreeMap<BaseId, usize>,
    buildables: Vec<BuildableDefinition>,
    buildable_index: BTreeMap<BuildableId, usize>,
    products: Vec<ProductDefinition>,
    product_index: BTreeMap<ProductId, usize>,
}

fn build_index<'a, Id>(ids: impl Iterator<Item = &'a Id>) -> Result<BTreeMap<Id, usize>, Id>
where
    Id: Clone + Ord + 'a,
{
    let mut index = BTreeMap::new();
    for (position, id) in ids.enumerate() {
        if index.insert(id.clone(), position).is_some() {
            return Err(id.clone());
        }
    }
    Ok(index)
}

impl Catalog {
    pub fn new(
        metadata: CatalogMetadata,
        default_base_id: BaseId,
        regions: Vec<RegionDefinition>,
        bases: Vec<BaseDefinition>,
        buildables: Vec<BuildableDefinition>,
        products: Vec<ProductDefinition>,
    ) -> Result<Self, CatalogValidationError> {
        if metadata.display_name().trim().is_empty() {
            return Err(CatalogValidationError::EmptyCatalogDisplayName);
        }
        for definition in &regions {
            if definition.display_name().trim().is_empty() {
                return Err(CatalogValidationError::EmptyRegionDisplayName(
                    definition.id().clone(),
                ));
            }
        }
        for definition in &bases {
            if definition.display_name().trim().is_empty() {
                return Err(CatalogValidationError::EmptyBaseDisplayName(
                    definition.id().clone(),
                ));
            }
        }
        for definition in &buildables {
            if definition.display_name().trim().is_empty() {
                return Err(CatalogValidationError::EmptyBuildableDisplayName(
                    definition.id().clone(),
                ));
            }
            if !(1..=4).contains(&definition.symbol().trim().chars().count()) {
                return Err(CatalogValidationError::InvalidBuildableSymbol(
                    definition.id().clone(),
                ));
            }
        }
        for definition in &products {
            if definition.display_name().trim().is_empty() {
                return Err(CatalogValidationError::EmptyProductDisplayName(
                    definition.id().clone(),
                ));
            }
        }

        let region_index = build_index(regions.iter().map(RegionDefinition::id))
            .map_err(CatalogValidationError::DuplicateRegionId)?;
        let base_index = build_index(bases.iter().map(BaseDefinition::id))
            .map_err(CatalogValidationError::DuplicateBaseId)?;
        if !base_index.contains_key(&default_base_id) {
            return Err(CatalogValidationError::MissingDefaultBase(default_base_id));
        }
        for definition in &bases {
            if !region_index.contains_key(definition.region_id()) {
                return Err(CatalogValidationError::MissingRegion {
                    base_id: definition.id().clone(),
                    region_id: definition.region_id().clone(),
                });
            }
        }
        let buildable_index = build_index(buildables.iter().map(BuildableDefinition::id))
            .map_err(CatalogValidationError::DuplicateBuildableId)?;
        let product_index = build_index(products.iter().map(ProductDefinition::id))
            .map_err(CatalogValidationError::DuplicateProductId)?;
        for definition in &buildables {
            let mut seen_targets = BTreeSet::new();
            for product_id in definition.production_targets() {
                if !seen_targets.insert(product_id) {
                    return Err(CatalogValidationError::DuplicateProductionTarget {
                        buildable_id: definition.id().clone(),
                        product_id: product_id.clone(),
                    });
                }
                if !product_index.contains_key(product_id) {
                    return Err(CatalogValidationError::MissingProductionTarget {
                        buildable_id: definition.id().clone(),
                        product_id: product_id.clone(),
                    });
                }
            }
        }

        Ok(Self {
            inner: Arc::new(CatalogData {
                metadata,
                default_base_id,
                regions,
                region_index,
                bases,
                base_index,
                buildables,
                buildable_index,
                products,
                product_index,
            }),
        })
    }

    pub fn metadata(&self) -> &CatalogMetadata {
        &self.inner.metadata
    }

    pub fn default_base_id(&self) -> &BaseId {
        &self.inner.default_base_id
    }

    pub fn default_base(&self) -> &BaseDefinition {
        self.base(self.default_base_id())
            .expect("validated catalog must contain its default base")
    }

    pub fn regions(&self) -> &[RegionDefinition] {
        &self.inner.regions
    }

    pub fn region(&self, id: &RegionId) -> Option<&RegionDefinition> {
        self.inner
            .region_index
            .get(id)
            .map(|position| &self.inner.regions[*position])
    }

    pub fn bases(&self) -> &[BaseDefinition] {
        &self.inner.bases
    }

    pub fn base(&self, id: &BaseId) -> Option<&BaseDefinition> {
        self.inner
            .base_index
            .get(id)
            .map(|position| &self.inner.bases[*position])
    }

    pub fn buildables(&self) -> &[BuildableDefinition] {
        &self.inner.buildables
    }

    pub fn buildable(&self, id: &BuildableId) -> Option<&BuildableDefinition> {
        self.inner
            .buildable_index
            .get(id)
            .map(|position| &self.inner.buildables[*position])
    }

    pub fn products(&self) -> &[ProductDefinition] {
        &self.inner.products
    }

    pub fn product(&self, id: &ProductId) -> Option<&ProductDefinition> {
        self.inner
            .product_index
            .get(id)
            .map(|position| &self.inner.products[*position])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCategory {
    Energy,
    ProductionI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockTemplate {
    XiranitePowerPole,
    RefineryUnit,
    CrushingUnit,
}

impl BlockTemplate {
    pub const ALL: [Self; 3] = [
        Self::XiranitePowerPole,
        Self::RefineryUnit,
        Self::CrushingUnit,
    ];

    pub fn definition(self) -> BlockDefinition {
        match self {
            Self::XiranitePowerPole => BlockDefinition {
                id: "xiranite_power_pole",
                display_name: "Xiranite Power Pole",
                category: BlockCategory::Energy,
                footprint: GridSize::new(2, 2)
                    .expect("catalog footprint dimensions must be positive"),
            },
            Self::RefineryUnit => BlockDefinition {
                id: "refinery_unit",
                display_name: "Refinery Unit",
                category: BlockCategory::ProductionI,
                footprint: GridSize::new(3, 3)
                    .expect("catalog footprint dimensions must be positive"),
            },
            Self::CrushingUnit => BlockDefinition {
                id: "crushing_unit",
                display_name: "Crushing Unit",
                category: BlockCategory::ProductionI,
                footprint: GridSize::new(3, 3)
                    .expect("catalog footprint dimensions must be positive"),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDefinition {
    id: &'static str,
    display_name: &'static str,
    category: BlockCategory,
    footprint: GridSize,
}

impl BlockDefinition {
    pub const fn id(self) -> &'static str {
        self.id
    }

    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    pub const fn category(self) -> BlockCategory {
        self.category
    }

    pub const fn footprint(self) -> GridSize {
        self.footprint
    }
}
