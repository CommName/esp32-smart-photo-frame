use std::sync::RwLock;

use image::{ImageBuffer, Rgb};

#[derive(Default)]
pub struct AppData {
    images: RwLock<Vec<ProccessedImage>>,
}

#[derive(Clone, Default)]
pub struct ProccessedImage {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}

impl AppData {
    pub fn get_random_image(&self) -> ProccessedImage {
        let images = self.images.read().unwrap();
        let index = rand::random_range(0..images.len());
        images[index].clone()
    }

    pub fn set_images(&self, images: Vec<ProccessedImage>) {
        *self.images.write().unwrap() = images;
    }
}

/*
     width
-----------------
|                | h
|      Left      | e
|                | i
|----------------| g
|                | h
|      Right     | t
|                |
-----------------
*/
impl From<ImageBuffer<Rgb<u8>, Vec<u8>>> for ProccessedImage {
    fn from(mut value: ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        let mut left_panel = Vec::new();
        let mut right_panel = Vec::new();

        value = image::imageops::rotate270(&value);

        for y in 0..value.height() {
            for x in (0..value.width()).step_by(2) {
                let pixel = value.get_pixel(x, y);
                let byte = map_color_to_byte(pixel);

                let pixel = value.get_pixel(x + 1, y);
                let byte = (byte << 4) | map_color_to_byte(pixel);

                if x < value.width() / 2 {
                    left_panel.push(byte);
                } else {
                    right_panel.push(byte);
                }
            }
        }

        ProccessedImage {
            left: left_panel,
            right: right_panel,
        }
    }
}

fn map_color_to_byte(color: &Rgb<u8>) -> u8 {
    match color {
        image::Rgb([0, 0, 0]) => 0,
        image::Rgb([255, 255, 255]) => 1,
        image::Rgb([255, 255, 0]) => 2,
        image::Rgb([255, 0, 0]) => 3,
        image::Rgb([0, 0, 255]) => 5,
        image::Rgb([0, 255, 0]) => 6,
        _ => unreachable!(),
    }
}
