use image::{ImageBuffer, Rgb, Rgba, imageops::ColorMap};

pub fn process_image(image: Vec<u8>) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    let mut img = image::load_from_memory(&image)?;

    // Make sure it in landscape orientation
    if img.height() > img.width() {
        img = img.rotate90();
    }
    let img = img.to_rgb8();

    let mut img = image::imageops::resize(&img, 1600, 1200, image::imageops::FilterType::Lanczos3);

    let color_map = Epd13in3ColorMap {
        colors: vec![
            image::Rgb([0, 0, 0]),
            image::Rgb([255, 0, 0]),
            image::Rgb([0, 255, 0]),
            image::Rgb([0, 0, 255]),
            image::Rgb([255, 255, 0]),
            image::Rgb([255, 255, 255]),
        ],
    };

    image::imageops::dither(&mut img, &color_map);
    // Placeholder implementation
    Ok(img)
}

struct Epd13in3ColorMap {
    colors: Vec<Rgb<u8>>,
}

impl ColorMap for Epd13in3ColorMap {
    type Color = image::Rgb<u8>;

    fn index_of(&self, pixel: &Self::Color) -> usize {
        self.colors
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| color_distance(pixel, c))
            .map(|(i, _)| i)
            .unwrap()
    }

    fn map_color(&self, color: &mut Self::Color) {
        let index = self.index_of(color);
        *color = self.colors[index].clone()
    }
}

fn color_distance(a: &Rgb<u8>, b: &Rgb<u8>) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    (dr * dr + dg * dg + db * db) as u32
}
