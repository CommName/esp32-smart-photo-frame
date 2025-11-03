#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

use smart_picture_frame::dev_config::DevConfig;
use smart_picture_frame::epd_13in3e::{Color, EPD13in3e};
use smart_picture_frame::image_data::{
    generate_color_blocks_static, generate_test_pattern_static, BMP_1,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize device configuration with GPIO pins
    let dev_config = DevConfig::new(
        Output::new(peripherals.GPIO13, Level::Low, OutputConfig::default()), // SCK
        Output::new(peripherals.GPIO14, Level::Low, OutputConfig::default()), // MOSI
        Output::new(peripherals.GPIO15, Level::High, OutputConfig::default()), // CS_M
        Output::new(peripherals.GPIO2, Level::High, OutputConfig::default()), // CS_S
        Output::new(peripherals.GPIO26, Level::Low, OutputConfig::default()), // RST
        Output::new(peripherals.GPIO27, Level::Low, OutputConfig::default()), // DC
        Input::new(
            peripherals.GPIO25,
            InputConfig::default().with_pull(Pull::Down),
        ), // BUSY
        Output::new(peripherals.GPIO33, Level::High, OutputConfig::default()), // PWR
    );

    // Create EPD instance
    let mut epd = EPD13in3e::new(dev_config);

    // Initialize and clear display
    epd.init();
    epd.clear(Color::White);

    // Wait
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(5000) {}

    // Display the BMP_1 image (partial display)
    epd.display_part(BMP_1, 400, 500, 400, 600);
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(9000) {}

    epd.module_exit();

    // Keep the program running
    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}
