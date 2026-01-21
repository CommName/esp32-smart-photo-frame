use image::ImageReader;
use poem_openapi::{OpenApi, payload::Binary};

use crate::image_processing;

const WIDTH: usize = 1200;
const HEIGHT: usize = 1600;

pub struct Api;

#[OpenApi]
impl Api {
    /// Hello world
    #[oai(path = "/next-picture", method = "get")]
    async fn next_picture(&self) -> Binary<Vec<u8>> {
        let image_data = ImageReader::open("./output.jpg")
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8()
            .into_raw();

        println!("Image data length: {}", image_data.len());
        let prepare = image_processing::prepeare_image_for_sending(&image_data, WIDTH, HEIGHT);
        let (left, right) = image_processing::split_into_left_right(&prepare, WIDTH, HEIGHT);

        let mut output = Vec::new();
        output.extend(left);
        output.extend(right);
        println!("Output data length: {}", output.len());
        Binary(output)
    }
}
