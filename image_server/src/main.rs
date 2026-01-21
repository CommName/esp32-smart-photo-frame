use poem::{EndpointExt, Route, Server, listener::TcpListener};
use poem_openapi::OpenApiService;

mod api_handlers;
mod esp_handler;
mod image_processing;
mod image_service;
mod immich;

pub use api_handlers::Api;

use crate::image_service::ImageService;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app_data = std::sync::Arc::new(ImageService::default());
    let app_data_clone = app_data.clone();
    tokio::spawn(async move {
        image_service::run_immich_fetcher_loop(app_data_clone).await;
    });
    let app_data_clone = app_data.clone();
    esp_handler::spawn_esp_server(app_data_clone).await;
    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000");
    // let ui = api_service.swagger_ui();

    let app = Route::new().nest("/", api_service).data(app_data);

    let _ = Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await;
}
