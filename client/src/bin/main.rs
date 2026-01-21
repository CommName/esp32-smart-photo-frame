#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::net::Ipv4Addr;

use esp_hal::rtc_cntl::Rtc;
use log::info;

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use smart_picture_frame::dev_config::DevConfig;
use smart_picture_frame::run;
use smoltcp::wire::IpAddress;
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

#[main]
fn main() -> ! {
    // Initialize println and logging support
    esp_println::logger::init_logger(log::LevelFilter::Info);

    // Both println! and log macros are now available
    println!("=== ESP32 Smart Photo Frame Starting ===");
    info!("Starting ESP32 Smart Photo Frame");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    info!("Initializing ESP32 HAL with 80 MHz CPU clock");
    let peripherals = esp_hal::init(config);
    info!("ESP32 HAL initialized successfully");

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);
    info!("Heap allocator initialized with 98767 bytes");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    info!("Timer group created");
    esp_rtos::start(timg0.timer0);
    info!("ESP-RTOS started");

    info!("Initializing Wi-Fi/BLE radio controller");
    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    info!("Radio controller initialized successfully");
    info!("Creating Wi-Fi controller and interfaces");
    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    info!("Wi-Fi controller created successfully");

    // Initialize device configuration with GPIO pins
    info!("Initializing device configuration with GPIO pins");
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
    info!("Device configuration initialized with GPIO pins");
    // Start the main application logic

    let mut rtc = Rtc::new(peripherals.LPWR);

    run(
        dev_config,
        wifi_controller,
        interfaces,
        SSID,
        IpAddress::Ipv4(Ipv4Addr::new(192, 168, 8, 4)),
        3000,
        PASSWORD,
        &mut rtc,
    )
}
