use crate::{block::BlockId, math::Vec3};

pub struct Chunk {
    blocks: [BlockId; Self::SIZE.x * Self::SIZE.y * Self::SIZE.z],
}

impl Chunk {
    pub const SIZE: Vec3<usize> = Vec3::new(16, 256, 16);

    pub fn flat() -> Self {
        let mut blocks = [BlockId::Air; Self::SIZE.x * Self::SIZE.y * Self::SIZE.z];
        for x in 0..Self::SIZE.x {
            for y in 0..Self::SIZE.y {
                for z in 0..Self::SIZE.z {
                    let index = Self::index(Vec3::new(x as i32, y as i32, z as i32)).unwrap();
                    blocks[index] = match y {
                        0..=32 => BlockId::Stone,
                        33..=254 => BlockId::Dirt,
                        255 => BlockId::Grass,
                        _ => BlockId::Air,
                    };
                }
            }
        }
        Self { blocks }
    }

    pub fn index(pos: Vec3<i32>) -> Option<usize> {
        if pos.is_any_negative() {
            return None;
        }
        let pos = pos.map(|x| x as usize);
        if pos.x >= Self::SIZE.x || pos.y >= Self::SIZE.y || pos.z >= Self::SIZE.z {
            return None;
        }
        Some(Self::SIZE.x * Self::SIZE.y * pos.z + Self::SIZE.x * pos.y + pos.x)
    }

    pub fn get(&self, pos: Vec3<i32>) -> Option<BlockId> {
        if pos.is_any_negative() {
            return None;
        }
        Self::index(pos).map(|index| self.blocks[index])
    }

    pub fn out_of_bounds(pos: Vec3<i32>) -> bool {
        pos.is_any_negative()
            || pos.x >= Self::SIZE.x as i32
            || pos.y >= Self::SIZE.y as i32
            || pos.z >= Self::SIZE.z as i32
    }
}
