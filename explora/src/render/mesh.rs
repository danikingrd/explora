use common::{
    chunk::Chunk,
    math::{Vec2, Vec3},
};

use super::{atlas::Atlas, Vertex};

pub fn create_chunk_mesh(chunk: &Chunk, mesh: &mut Vec<Vertex>, pos: Vec2<i32>, atlas: &Atlas) {
    for x in 0..Chunk::SIZE.x {
        for y in 0..Chunk::SIZE.y {
            for z in 0..Chunk::SIZE.z {
                let origin = Vec3::new(x, y, z).as_::<i32>();
                let block = chunk.get(origin).unwrap();
                let offset = Vec3::new(
                    pos.x as f32 * Chunk::SIZE.x as f32 + x as f32,
                    y as f32,
                    pos.y as f32 * Chunk::SIZE.z as f32 + z as f32,
                );
                let texture = atlas.block_texture(block);
                // North
                if Chunk::out_of_bounds(origin + Vec3::unit_z()) {
                    let north = texture.values[0];
                    mesh.push(Vertex::new(
                        Vec3::unit_x() + Vec3::unit_y() + Vec3::unit_z() + offset,
                        north,
                    ));
                    mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_z() + offset, north));
                    mesh.push(Vertex::new(Vec3::zero() + Vec3::unit_z() + offset, north));
                    mesh.push(Vertex::new(Vec3::unit_y() + Vec3::unit_z() + offset, north));
                }

                // South
                if Chunk::out_of_bounds(origin - Vec3::unit_z()) {
                    let south = texture.values[1];
                    mesh.push(Vertex::new(Vec3::unit_y() + offset, south));
                    mesh.push(Vertex::new(Vec3::zero() + offset, south));
                    mesh.push(Vertex::new(Vec3::unit_x() + offset, south));
                    mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_y() + offset, south));
                }

                // East
                if Chunk::out_of_bounds(origin + Vec3::unit_x()) {
                    let east = texture.values[2];
                    mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_y() + offset, east));
                    mesh.push(Vertex::new(Vec3::unit_x() + offset, east));
                    mesh.push(Vertex::new(Vec3::unit_x() + Vec3::unit_z() + offset, east));
                    mesh.push(Vertex::new(
                        Vec3::unit_x() + Vec3::unit_z() + Vec3::unit_y() + offset,
                        east,
                    ));
                }
                // West
                if Chunk::out_of_bounds(origin - Vec3::unit_x()) {
                    let west = texture.values[3];
                    mesh.push(Vertex::new(Vec3::unit_z() + Vec3::unit_y() + offset, west));
                    mesh.push(Vertex::new(Vec3::unit_z() + offset, west));
                    mesh.push(Vertex::new(Vec3::zero() + offset, west));
                    mesh.push(Vertex::new(Vec3::unit_y() + offset, west));
                }

                if Chunk::out_of_bounds(origin + Vec3::unit_y()) {
                    // Top
                    let top = texture.values[4];
                    mesh.push(Vertex::new(Vec3::unit_z() + Vec3::unit_y() + offset, top));
                    mesh.push(Vertex::new(Vec3::unit_y() + offset, top));
                    mesh.push(Vertex::new(Vec3::unit_y() + Vec3::unit_x() + offset, top));
                    mesh.push(Vertex::new(
                        Vec3::unit_y() + Vec3::unit_x() + Vec3::unit_z() + offset,
                        top,
                    ));
                }

                if Chunk::out_of_bounds(origin - Vec3::unit_y()) {
                    // Bottom
                    let bottom = texture.values[5];
                    mesh.push(Vertex::new(Vec3::zero() + offset, bottom));
                    mesh.push(Vertex::new(Vec3::unit_z() + offset, bottom));
                    mesh.push(Vertex::new(
                        Vec3::unit_z() + Vec3::unit_x() + offset,
                        bottom,
                    ));
                    mesh.push(Vertex::new(Vec3::unit_x() + offset, bottom));
                }
            }
        }
    }
}
