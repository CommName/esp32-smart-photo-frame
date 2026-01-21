use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::image_service::ImageService;

pub async fn spawn_esp_server(app_data: std::sync::Arc<ImageService>) {
    tokio::spawn(async move {
        esp_server(app_data).await;
    });
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

async fn esp_server(app_data: std::sync::Arc<ImageService>) {
    println!("Starting ESP server on port 4000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();

    loop {
        println!("Waiting for connection...");
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New connection from {}", socket.peer_addr().unwrap());

        // Handle the connection (read/write data)
        // For example, you can read data from the socket
        let app_data = app_data.clone();
        let _ = tokio::spawn(async move {
            let current_image = app_data.get_image().await;
            let mut data_sent = 0;
            let mut buf = [0u8; 1024];
            socket.read(&mut buf).await.unwrap(); // Wait for initial data

            println!("Sending left data [{} bytes]", current_image.left.len());
            send_data_through_socket(&mut socket, &current_image.left, &mut data_sent).await;

            println!("Sending right data [{} bytes]", current_image.right.len());
            send_data_through_socket(&mut socket, &current_image.right, &mut data_sent).await;

            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            println!("Sent {} bytes to {}", data_sent, addr);
        });
    }
}
