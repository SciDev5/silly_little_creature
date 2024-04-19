use std::io::Cursor;

use image::io::Reader as ImageReader;

use crate::util::Vec2I;

use super::glrs;

pub struct ImageAsset {
    data: Vec<glrs::GLTexPixel>,
    dims: Vec2I,
}
impl ImageAsset {
    pub fn decode_bytes(bytes: &[u8]) -> Self {
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        Self {
            dims: Vec2I::new(img.width() as i32, img.height() as i32),
            data: img
                .into_rgba8()
                .chunks_exact(4)
                .map(|v| glrs::GLTexPixel {
                    r: v[0],
                    g: v[1],
                    b: v[2],
                    a: v[3],
                })
                .collect::<Vec<_>>(),
        }
    }
    pub fn pixels(&self) -> &[glrs::GLTexPixel] {
        &self.data
    }
    pub fn get_dimensions(&self) -> Vec2I {
        self.dims
    }
}

#[macro_export]
macro_rules! include_imageasset {
    ($file:expr $(,)?) => {
        $crate::render::image_asset::ImageAsset::decode_bytes(include_bytes!($file))
    };
}
