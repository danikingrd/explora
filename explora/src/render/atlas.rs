use std::path::Path;

use crate::render::png_utils;

use super::png_utils::PngImage;

pub struct Atlas {
    // TODO: Temporal.
    pub image: PngImage,
    pub tile_size: usize,
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

        // the number of tiles per row/column
        let atlas_tile_count = (files.len() as f32).sqrt().ceil() as usize;
        // tracing::info!(?atlas_tile_count);

        // We need to know what the size of each individual tile is.
        // We can get this from the first texture, assuming they are all the same size.
        let first_image = png_utils::read(&files[0]).unwrap();
        let atlas_width = first_image.width as usize * atlas_tile_count;
        let atlas_height = first_image.height as usize * atlas_tile_count;
        let mut pixels = vec![0; atlas_width * atlas_height * 4];

        tracing::info!(?atlas_tile_count, ?atlas_width, ?atlas_height, ?first_image.width, ?first_image.height);

        let mut id = 0;
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
        })
    }
}
