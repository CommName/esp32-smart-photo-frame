#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::net::Ipv4Addr;

use alloc::vec::Vec;
use log::{debug, info, warn};

use blocking_network_stack::Stack;
use embedded_io::Read;
use embedded_io::Write;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::rng::Rng;
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{main, time};
use esp_println::println;
use smart_picture_frame::dev_config::DevConfig;
use smart_picture_frame::epd_13in3e::{Color, EPD_HEIGHT, EPD_WIDTH, EPD13in3e};
use smart_picture_frame::wifi::*;
use smoltcp::iface::{SocketSet, SocketStorage};
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

const STATIC_IP: &str = "192.168.8.201";
const GATEWAY_IP: &str = "192.168.8.1";
const SSID: &str = "WIFI";
const PASSWORD: &str = "PASSWORD";

#[main]
fn main() -> ! {
    // Initialize println and logging support
    esp_println::logger::init_logger(log::LevelFilter::Debug);

    // Both println! and log macros are now available
    println!("=== ESP32 Smart Photo Frame Starting ===");
    info!("Starting ESP32 Smart Photo Frame");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    info!("Initializing ESP32 HAL with max CPU clock");
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
    let (mut wifi_controller, interfaces) =
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

    info!("Connecting to Wi-Fi network");
    connect_wifi(&mut wifi_controller, SSID, PASSWORD);
    info!("Wi-Fi connection established");

    // Create API call
    info!("Setting up network interface");
    let mut device = interfaces.sta;
    let iface = create_interface(&mut device);
    info!("Network interface created");

    info!("Disabling Wi-Fi power saving mode");
    wifi_controller
        .set_power_saving(esp_radio::wifi::PowerSaveMode::None)
        .unwrap();
    info!("Wi-Fi power saving disabled");

    info!("Creating socket set with 3 entries");
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let socket_set = SocketSet::new(&mut socket_set_entries[..]);

    info!("Initializing RNG and network stack");
    let rng = Rng::new();
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let mut stack = Stack::new(iface, device, socket_set, now, rng.random());
    info!("Network stack initialized");

    info!(
        "Configuring network interface with static IP: {}, Gateway: {}",
        STATIC_IP, GATEWAY_IP
    );
    stack
        .set_iface_configuration(&blocking_network_stack::ipv4::Configuration::Client(
            blocking_network_stack::ipv4::ClientConfiguration::Fixed(
                blocking_network_stack::ipv4::ClientSettings {
                    ip: blocking_network_stack::ipv4::Ipv4Addr::from(parse_ip(STATIC_IP)),
                    subnet: blocking_network_stack::ipv4::Subnet {
                        gateway: blocking_network_stack::ipv4::Ipv4Addr::from(parse_ip(GATEWAY_IP)),
                        mask: blocking_network_stack::ipv4::Mask(24),
                    },
                    dns: None,
                    secondary_dns: None,
                },
            ),
        ))
        .unwrap();
    info!("Network interface configured successfully");

    info!("Allocating buffers - RX: 1536 bytes, TX: 1536 bytes, Image: 240000 bytes");
    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];

    // Initialize and clear display
    // Create EPD instance
    info!("Creating EPD 13.3 inch e-paper display instance");
    let mut epd = EPD13in3e::new(dev_config);
    info!("Initializing e-paper display");
    epd.init();

    // Wait
    info!("Waiting 5 seconds for display to settle");
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(5000) {}

    info!("Creating socket from network stack");
    let mut socket = stack.get_socket(&mut rx_buffer, &mut tx_buffer);
    socket.work();
    info!("Socket created and initialized");

    info!("Opening connection to server 192.168.8.8:3000");
    socket
        .open(IpAddress::Ipv4(Ipv4Addr::new(192, 168, 8, 8)), 3000)
        .unwrap();
    info!("Connection established");

    info!("Sending HTTP request: GET /next-picture");
    socket.write(b"GET /next-picture HTTP/1.0\r\n\r\n").unwrap();
    socket.flush().unwrap();
    info!("HTTP request sent");

    info!("Starting to receive image data");
    let mut bytes_read = 0;
    let mut header_read = false;
    let mut left_size = true;
    loop {
        let mut response_buffer = [0u8; 1024];
        let Ok(response) = socket.read(&mut response_buffer) else {
            break;
        };
        debug!("Received {} bytes from server", response);

        if !header_read {
            // Simple header parsing to skip HTTP headers
            if let Some(header_end) = response_buffer[..response]
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
            {
                info!("HTTP headers parsed, starting image data reception");
                let image_start = header_end + 4;
                let image_bytes = response - image_start;
                epd.set_right_panel();
                epd.send_data_bytes(&response_buffer[image_start..response]);
                //image_data.extend_from_slice(&response_buffer[image_start..response]);
                bytes_read += image_bytes;
                header_read = true;
                debug!("First image chunk: {} bytes", image_bytes);
            }
        } else {
            let limit = EPD_WIDTH as usize * EPD_HEIGHT as usize / 4;
            if left_size && (bytes_read + response >= limit) {
                epd.send_data_bytes(&response_buffer[..limit - bytes_read]);

                epd.set_left_panel();
                left_size = false;
            } else {
                epd.send_data_bytes(&response_buffer[..response]);
                bytes_read += response;
            }

            debug!("Total image data received: {} bytes", bytes_read);
        }

        if response == 0 {
            break;
        }
    }

    epd.cs_all(true);
    epd.turn_on_display();

    info!("Waiting 5 seconds for display to settle");
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(5000) {}

    info!("Shutting down e-paper display module");
    epd.module_exit();
    // Keep the program running
    info!("Entering main loop");
    loop {
        debug!("Main loop iteration");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
    }
}
