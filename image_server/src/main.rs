use std::{io::Cursor, sync::Arc};

use ::image::ImageReader;
use image::{ImageBuffer, Rgb};
use poem::{
    EndpointExt, Route, Server,
    listener::{self, TcpListener},
    web::Data,
};
use poem_openapi::{
    OpenApi, OpenApiService,
    payload::{Binary, EventStream, PlainText},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

mod image_processing;

struct Api;

const WIDTH: usize = 1200;
const HEIGHT: usize = 1600;

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

    #[oai(path = "/picture", method = "post")]
    async fn upload_picture(
        &self,
        image: Binary<Vec<u8>>,
        data: Data<&Arc<AppData>>,
    ) -> PlainText<String> {
        // Decode the image
        let mut image_data: Vec<u8> = image.0;
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

        data.insert_image(resized_image.clone()).await;

        resized_image.save("./output.jpg").unwrap();

        PlainText("Image processed".to_string())
    }
}

#[derive(Default)]
pub struct AppData {
    pub current_image: RwLock<CurrentImage>,
}

#[derive(Default)]
pub struct CurrentImage {
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}

impl AppData {
    pub async fn insert_image(&self, data: ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let mut current_image = self.current_image.write().await;
        let data = image_processing::prepeare_image_for_sending(&data.into_raw(), WIDTH, HEIGHT);
        let (left, right) = image_processing::split_into_left_right(&data, WIDTH, HEIGHT);
        let left = pack_vec(&left);
        let right = pack_vec(&right);
        current_image.left = left;
        current_image.right = right;
    }

    pub async fn get_image(&self) -> (Vec<u8>, Vec<u8>) {
        let current_image = self.current_image.read().await;
        (current_image.left.clone(), current_image.right.clone())
    }
}

async fn send_data_through_socket(
    socket: &mut tokio::net::TcpStream,
    data: &[u8],
    data_sent: &mut usize,
) {
    let mut buf = [0u8; 1024];
    for chunk in data.chunks(1024) {
        if let Err(e) = socket.write(chunk).await {
            eprintln!("Failed to send data to: {e}");
            return;
        }
        socket.flush().await.unwrap();
        *data_sent += chunk.len();

        // println!("Sent {} bytes so far", data_sent);

        socket.read(&mut buf).await.unwrap();
    }
}

async fn esp_server(app_data: std::sync::Arc<AppData>) {
    println!("Starting ESP server on port 4000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();

    loop {
        println!("Waiting for connection...");
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New connection from {}", socket.peer_addr().unwrap());

        // Handle the connection (read/write data)
        // For example, you can read data from the socket
        let app_data = app_data.clone();
        let x = tokio::spawn(async move {
            let (left, right) = app_data.get_image().await;
            let mut data_sent = 0;
            let mut buf = [0u8; 1024];
            socket.read(&mut buf).await.unwrap(); // Wait for initial data

            let white_data = vec![0xFFu8; right.len()];

            println!("Sending left data [{} bytes]", left.len());
            send_data_through_socket(&mut socket, &&left[..left.len() / 2], &mut data_sent).await;
            send_data_through_socket(&mut socket, &white_data[..left.len() / 2], &mut data_sent)
                .await;

            // let right_data = &right[..right.len() / 4];
            println!("Sending right data [{} bytes]", right.len());
            send_data_through_socket(&mut socket, &right[..right.len() / 2], &mut data_sent).await;
            send_data_through_socket(&mut socket, &white_data[..right.len() / 2], &mut data_sent)
                .await;

            // send_data_through_socket(&mut socket, &right[..right.len() / 4], &mut data_sent).await;

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            println!("Sent {} bytes to {}", data_sent, addr);
        });
    }
}

#[tokio::main]
async fn main() {
    let app_data = std::sync::Arc::new(AppData::default());
    let app_data_clone = app_data.clone();
    tokio::spawn(async move {
        esp_server(app_data_clone).await;
    });
    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000");
    // let ui = api_service.swagger_ui();

    let app = Route::new().nest("/", api_service).data(app_data);

    let _ = Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await;
}
