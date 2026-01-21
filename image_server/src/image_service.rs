use std::io::Cursor;

use image::ImageReader;
use tokio::sync::Mutex;

use crate::{image_processing, immich};

const WIDTH: usize = 1200;
const HEIGHT: usize = 1600;

#[derive(Default)]
pub struct ImageService {
    proccessed_images: Mutex<ImageHolder>,
}

#[derive(Default)]
struct ImageHolder {
    pub processed_images: Vec<CurrentImage>,
    pub current_index: usize,
}

impl ImageHolder {
    fn get_image(&mut self) -> CurrentImage {
        if self.processed_images.is_empty() {
            return CurrentImage {
                left: vec![],
                right: vec![],
            };
        }
        let image = self.processed_images[self.current_index].clone();
        self.current_index = (self.current_index + 1) % self.processed_images.len();
        image
    }

    fn insert_image(&mut self, image_data: &[u8]) {
        let mut buffer = Cursor::new(image_data);
        let image = ImageReader::new(&mut buffer).with_guessed_format().unwrap();
        let image = image.decode().unwrap().to_rgb8();

        let mut resized_image = image::imageops::resize(
            &image,
            WIDTH as u32,
            HEIGHT as u32,
            image::imageops::FilterType::Triangle,
        );

        let color_pallet = vec![
            image::Rgb([0, 0, 0]),
            image::Rgb([255, 255, 255]),
            image::Rgb([255, 255, 0]),
            image::Rgb([255, 0, 0]),
            image::Rgb([0, 0, 255]),
            image::Rgb([0, 255, 0]),
        ];
        image_processing::dither_with_palette(&mut resized_image, &color_pallet);

        let data =
            crate::image_processing::prepeare_image_for_sending(&resized_image, WIDTH, HEIGHT);
        let (left, right) = crate::image_processing::split_into_left_right(&data, WIDTH, HEIGHT);
        let left = pack_vec(&left);
        let right = pack_vec(&right);
        self.processed_images.push(CurrentImage { left, right });
    }
}

#[derive(Default, Clone)]
pub struct CurrentImage {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}

impl ImageService {
    async fn insert_image_data(&self, set_processed_images: ImageHolder) {
        *self.proccessed_images.lock().await = set_processed_images;
    }

    pub async fn get_image(&self) -> CurrentImage {
        self.proccessed_images.lock().await.get_image()
    }
}

pub async fn run_immich_fetcher_loop(app_data: std::sync::Arc<ImageService>) {
    let immich_token =
        std::env::var("IMMICH_TOKEN").expect("IMMICH_TOKEN must be set in .env file");
    let immich_url = std::env::var("IMMICH_URL").expect("IMMICH_URL must be set in .env file");
    let immich_album_id =
        std::env::var("IMMICH_ALBUM_ID").expect("IMMICH_ALBUM_ID must be set in .env file");

    let immich_api = immich::ImmichApi::new(immich_token, immich_url, immich_album_id);
    loop {
        let album_data = immich_api
            .get_album_with_assets(&uuid::Uuid::parse_str(&immich_api.album_id).unwrap())
            .await;

        let mut image_holder = ImageHolder::default();

        for photo in album_data.assets.iter() {
            if let immich::AssetType::Image = photo.asset_type {
                let image_data = immich_api.download_asset(&photo.id).await;
                image_holder.insert_image(&image_data);
            }
        }

        app_data.insert_image_data(image_holder).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;
    }
    // Placeholder for future implementation
}

fn pack_vec(left: &[u8]) -> Vec<u8> {
    let mut pxInd = 0;
    let mut packed: Vec<u8> = Vec::new();
    while pxInd < left.len() {
        // Process each pixel
        let mut v = 0;
        for i in (0..8).step_by(4) {
            if pxInd < left.len() {
                v |= (left[pxInd]) << i;
                pxInd += 1;
            }
        }
        packed.push(v);
    }
    packed
}
