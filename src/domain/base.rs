use super::geometry::GridSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseKind {
    Main,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryLevel {
    Standard,
    AreaExpansionI,
    AreaExpansionII,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseTemplate {
    MainCurrent,
    Secondary(SecondaryLevel),
}

impl BaseTemplate {
    pub const ALL: [Self; 4] = [
        Self::MainCurrent,
        Self::Secondary(SecondaryLevel::Standard),
        Self::Secondary(SecondaryLevel::AreaExpansionI),
        Self::Secondary(SecondaryLevel::AreaExpansionII),
    ];

    pub const fn kind(self) -> BaseKind {
        match self {
            Self::MainCurrent => BaseKind::Main,
            Self::Secondary(_) => BaseKind::Secondary,
        }
    }

    pub fn bounds(self) -> GridSize {
        let side = match self {
            Self::MainCurrent => 80,
            Self::Secondary(SecondaryLevel::Standard) => 30,
            Self::Secondary(SecondaryLevel::AreaExpansionI) => 40,
            Self::Secondary(SecondaryLevel::AreaExpansionII) => 50,
        };

        GridSize::new(side, side).expect("base template dimensions must be positive")
    }
}
