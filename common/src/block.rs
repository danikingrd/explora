#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlockId {
    Air,
    Dirt,
    Grass,
    Stone,
}

impl BlockId {
    pub const fn is_air(self) -> bool {
        matches!(self, Self::Air)
    }

    pub const fn is_solid(self) -> bool {
        !self.is_air()
    }
}
