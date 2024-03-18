use std::{collections::HashMap, path::Path};

use common::block::BlockId;

use crate::render::png_utils;

use super::png_utils::PngImage;

pub struct BlockTexture {
    // 0 - North
    // 1 - South
    // 2 - East
    // 3 - West
    // 4 - Top
    // 5 - Bottom
    pub values: [u32; 6],
}

pub struct Atlas {
    // TODO: Temporal.
    pub image: PngImage,
    pub tile_size: usize,
    textures: HashMap<String, u32>,
}

impl Atlas {
    pub fn block_texture(&self, id: BlockId) -> BlockTexture {
        // TODO: Temporaal
        match id {
            BlockId::Dirt => {
                let id = self.get("dirt");
                BlockTexture {
                    values: [id, id, id, id, id, id],
                }
            }
            BlockId::Grass => {
                let top = self.get("grass_top");
                let side = self.get("grass_side");
                let bottom = self.get("dirt");
                BlockTexture {
                    values: [side, side, side, side, top, bottom],
                }
            }
            BlockId::Stone => {
                let id = self.get("stone");
                BlockTexture {
                    values: [id, id, id, id, id, id],
                }
            }
            _ => {
                let id = self.get("default");
                BlockTexture {
                    values: [id, id, id, id, id, id],
                }
            }
        }
    }
    pub fn get(&self, name: &str) -> u32 {
        match self.textures.get(name) {
            Some(id) => *id,
            None => {
                tracing::warn!("Texture not found: {}", name);
                0
            }
        }
    }
}

#[derive(Debug)]
pub enum AtlasError {
    Io(std::io::ErrorKind),
}

impl From<std::io::Error> for AtlasError {
    fn from(value: std::io::Error) -> Self {
        AtlasError::Io(value.kind())
    }
}

impl Atlas {
    pub fn pack_textures<P: AsRef<Path>>(resource: P) -> Result<Self, AtlasError> {
        let files = std::fs::read_dir(&resource)?
            .map(|x| x.map(|x| x.path()))
            // filter out anything that does not contain a png
            .filter(|x| {
                x.as_ref()
                    .unwrap()
                    .extension()
                    .map(|x| x == "png")
                    .unwrap_or(false)
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        tracing::info!(?files);
        // the number of tiles per row/column
        let atlas_tile_count = ((files.len() + 1) as f32).sqrt().ceil() as usize;
        tracing::info!(?atlas_tile_count);

        // We need to know what the size of each individual tile is.
        // We can get this from the first texture, assuming they are all the same size.
        let first_image = png_utils::read(&files[0]).unwrap();
        let atlas_width = first_image.width as usize * atlas_tile_count;
        let atlas_height = first_image.height as usize * atlas_tile_count;
        let mut pixels = vec![0; atlas_width * atlas_height * 4];

        draw_default_texture(
            first_image.width,
            first_image.height,
            atlas_width,
            &mut pixels,
        );

        tracing::info!(?atlas_tile_count, ?atlas_width, ?atlas_height, ?first_image.width, ?first_image.height);
        let mut textures = HashMap::new();
        textures.insert("default".to_owned(), 0);

        let mut id = 1;
        for path in &files {
            if path.is_dir() {
                continue; // skip just for now
            }
            let Ok(image) = png_utils::read(path) else {
                tracing::warn!("Failed to read texture at {}", path.display());
                continue;
            };

            if image.width != first_image.width || image.height != first_image.height {
                tracing::warn!(
                    "Found texture with invalid size: {}x{} (expected {}x{})",
                    image.width,
                    image.height,
                    first_image.width,
                    first_image.height
                );
                continue;
            }
            tracing::info!("Packing texture... id={} path={}", id, path.display());

            let pixel_x = (id % atlas_tile_count) * image.width as usize;
            let pixel_y = (id / atlas_tile_count) * image.height as usize;

            for y in 0..image.height as usize {
                for x in 0..image.width as usize {
                    let index = (y * image.width as usize + x) * image.channels as usize;
                    let atlas_index = ((pixel_y + y) * atlas_width + pixel_x + x) * 4;

                    pixels[atlas_index..atlas_index + 4]
                        .copy_from_slice(&image.pixels[index..index + 4]);
                }
            }
            let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
            textures.insert(name, id as u32);
            id += 1;
        }

        // TODO: Temporal.
        png_utils::write(
            "atlas.png",
            &pixels,
            atlas_width as u32,
            atlas_height as u32,
        )
        .unwrap();
        Ok(Self {
            image: PngImage {
                width: atlas_width as u32,
                height: atlas_height as u32,
                pixels,
                channels: 4,
            },
            tile_size: first_image.width as usize,
            textures,
        })
    }
}

fn draw_default_texture(tile_width: u32, tile_height: u32, atlas_width: usize, atlas: &mut [u8]) {
    tracing::info!("Drawing default texture {}x{}", tile_width, tile_height);
    for y in 0..tile_height as usize {
        for x in 0..tile_width as usize {
            let atlas_index = ((y) * atlas_width + x) * 4;
            if (x / 8 + y / 8) % 2 == 0 {
                atlas[atlas_index..atlas_index + 4].copy_from_slice(&[0, 0, 0, 255]);
            } else {
                atlas[atlas_index..atlas_index + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
}
