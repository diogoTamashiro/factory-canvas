#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPoint {
    pub x: i32,
    pub y: i32,
}

impl GridPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    width: u16,
    height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridSizeError {
    ZeroWidth,
    ZeroHeight,
}

impl GridSize {
    pub const fn new(width: u16, height: u16) -> Result<Self, GridSizeError> {
        if width == 0 {
            return Err(GridSizeError::ZeroWidth);
        }
        if height == 0 {
            return Err(GridSizeError::ZeroHeight);
        }

        Ok(Self { width, height })
    }

    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    Zero,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

impl Rotation {
    pub const fn clockwise(self) -> Self {
        match self {
            Self::Zero => Self::Clockwise90,
            Self::Clockwise90 => Self::Clockwise180,
            Self::Clockwise180 => Self::Clockwise270,
            Self::Clockwise270 => Self::Zero,
        }
    }

    pub const fn apply_to(self, footprint: GridSize) -> GridSize {
        match self {
            Self::Zero | Self::Clockwise180 => footprint,
            Self::Clockwise90 | Self::Clockwise270 => GridSize {
                width: footprint.height,
                height: footprint.width,
            },
        }
    }
}
