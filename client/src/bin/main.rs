#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::net::Ipv4Addr;
use core::str::FromStr;

use client::dev_config::DevConfig;
use client::epd13in3;
use embedded_io::{Read, Write};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use log::info;
use smoltcp::iface::SocketStorage;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.0

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");

    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut stack =
        client::network::create_stack(peripherals.WIFI, &mut socket_set_entries, &radio_init);

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
    log::info!("Clearing the e-paper display...");
    let mut epd = client::epd13in3::EPD13in3e::new(dev_config);

    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let server_address =
        smoltcp::wire::IpAddress::Ipv4(Ipv4Addr::from_str(env!("SERVER_ADDRESS")).unwrap());
    let server_port: u16 = env!("SERVER_PORT").parse().unwrap();

    let response = b"Ok";
    let number_of_pixels_in_bytes = (epd13in3::EPD_HEIGHT * epd13in3::EPD_WIDTH / 2) as usize;
    let mut socket = stack.get_socket(&mut rx_buffer, &mut tx_buffer);
    loop {
        info!("Opening socket to server...");
        socket.work();
        if let Err(e) = socket.open(server_address, server_port) {
            log::error!("Failed to open socket: {:?}", e);
            continue;
        }

        log::info!("Connecting socket to server...");

        while !socket.is_connected() {
            socket.work();
        }
        log::info!("Socket connected to server.");

        epd.init();
        epd.select_left_panel();

        let mut buffer_read = 0;
        let mut buff_reader = [0u8; 4096];
        loop {
            match socket.read(&mut buff_reader) {
                Ok(0) => {
                    // EOF
                    break;
                }
                Ok(n) => {
                    if n + buffer_read < (number_of_pixels_in_bytes / 2)
                        || buffer_read >= (number_of_pixels_in_bytes / 2)
                    {
                        epd.send_data_bytes(&buff_reader[..n]);
                    } else if n + buffer_read == (number_of_pixels_in_bytes / 2) {
                        epd.send_data_bytes(&buff_reader[..n]);
                        epd.select_right_panel();
                    } else {
                        epd.send_data_bytes(
                            &buff_reader[..(number_of_pixels_in_bytes / 2 - buffer_read)],
                        );
                        epd.select_right_panel();
                        epd.send_data_bytes(
                            &buff_reader[(number_of_pixels_in_bytes / 2 - buffer_read)..n],
                        );
                    }
                    // epd.send_data_bytes(&buff_reader[..n]);
                    buffer_read += n;
                    if buffer_read == number_of_pixels_in_bytes {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Socket read error: {:?}", e);
                    break;
                }
            }
            if let Err(e) = socket.write(response) {
                log::error!("Socket write error: {:?}", e);
                break;
            }
        }

        info!("Hello world!");
        let delay_start = Instant::now();

        socket.close();
        while delay_start.elapsed() < Duration::from_minutes(5) {}
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
