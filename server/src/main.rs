use std::{cmp::min, str::FromStr, sync::Arc};

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use uuid::Uuid;

use crate::{
    app_data::{AppData, ProccessedImage},
    image_ops::process_image,
    immich::Immich,
};

mod app_data;
mod image_ops;
mod immich;

pub async fn send_buffer(socket: &mut TcpStream, buffer: &[u8]) -> tokio::io::Result<()> {
    let mut _read_buff = [0u8; 1024];

    for chunk in buffer.chunks(500) {
        socket.write_all(chunk).await?;

        socket.flush().await?;
        let n = socket.read(&mut _read_buff).await?;
        if n != 2 {}
    }
    Ok(())
}

pub async fn send_photo(mut socket: tokio::net::TcpStream, photo: ProccessedImage) {
    // let panel = [0x00u8; 1200 * 1600 / 4];
    println!("Left panel size: {} bytes", photo.left.len());
    send_buffer(&mut socket, &photo.left).await.unwrap();
    println!("Right panel size: {} bytes", photo.right.len());
    send_buffer(&mut socket, &photo.right).await.unwrap();
}

pub async fn esp_server(app_data: Arc<AppData>) {
    println!("Starting server on port 2025...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2025").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        println!("Client connected: {:?}", socket.peer_addr());
        let photo = app_data.get_random_image();
        tokio::spawn(async move {
            send_photo(socket, photo).await;
        });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    const IMMICH_SERVER: &str = env!("IMMICH_SERVER");
    const IMMICH_TOKEN: &str = env!("IMMICH_TOKEN");
    const IMMICH_ALBUM: &str = env!("IMMICH_ALBUM");

    let immich_album = Uuid::from_str(IMMICH_ALBUM).unwrap();

    let image_api = Immich::new(IMMICH_SERVER.to_string(), IMMICH_TOKEN.to_string());

    println!("Fetching photos from Immich...");
    let images = image_api
        .get_photos(immich_album)
        .await?
        .into_iter()
        .enumerate()
        .map(|(index, bytes)| {
            let image = process_image(bytes).unwrap();
            ProccessedImage::from(image)
        })
        .collect();

    println!("Initialization complete, starting server...");
    let app_data = Arc::new(AppData::default());
    app_data.set_images(images);
    esp_server(Arc::clone(&app_data)).await;

    Ok(())
}
