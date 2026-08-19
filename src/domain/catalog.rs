use super::geometry::GridSize;

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
                display_name: "Poste de Xiranita",
                category: BlockCategory::Energy,
                footprint: GridSize::new(2, 2)
                    .expect("catalog footprint dimensions must be positive"),
            },
            Self::RefineryUnit => BlockDefinition {
                id: "refinery_unit",
                display_name: "Unidade de Refinaria",
                category: BlockCategory::ProductionI,
                footprint: GridSize::new(3, 3)
                    .expect("catalog footprint dimensions must be positive"),
            },
            Self::CrushingUnit => BlockDefinition {
                id: "crushing_unit",
                display_name: "Unidade de Trituração",
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
