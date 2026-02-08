use std::{cmp::min, str::FromStr};

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use uuid::Uuid;

use crate::{image_ops::process_image, immich::Immich};

mod image_ops;
mod immich;

pub async fn send_buffer(socket: &mut TcpStream, buffer: &[u8]) -> tokio::io::Result<()> {
    let mut bytes_sent = 0;
    let mut read_buff = [0u8; 1024];
    let total_bytes = buffer.len();

    while bytes_sent < total_bytes {
        let bytes_to_send = min(1024, total_bytes - bytes_sent);

        let n = socket
            .write(&buffer[bytes_sent..(bytes_sent + bytes_to_send)])
            .await?;
        bytes_sent += n;

        socket.flush().await?;
        socket.read(&mut read_buff).await?;
    }
    Ok(())
}

pub async fn send_photo(mut socket: tokio::net::TcpStream) {
    let panel = [0x00u8; 1200 * 1600 / 4];

    send_buffer(&mut socket, &panel).await.unwrap();
    send_buffer(&mut socket, &panel).await.unwrap();
}

pub async fn esp_server() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2025").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            send_photo(socket).await;
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

    let images = image_api.get_photos(immich_album).await?;
    for (index, image) in images.into_iter().enumerate() {
        let image = process_image(image)?;
        image.save(format!("./images/{index}.jpg"))?;
    }
    esp_server().await;

    Ok(())
}
