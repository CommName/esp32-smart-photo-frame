#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::net::Ipv4Addr;
use core::str::FromStr;

use blocking_network_stack::Stack;
use client::dev_config::DevConfig;
use client::epd13in3;
use embedded_io::{Read, Write};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::rtc_cntl::sleep::TimerWakeupSource;
use esp_hal::time::{Duration, Instant};
use esp_hal::timer::timg::TimerGroup;
use log::info;
use smoltcp::{
    iface::{SocketSet, SocketStorage},
    wire::DhcpOption,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use client::network::*;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // generator version: 1.0.0
    let refresh_rate = core::time::Duration::from_mins(env!("REFRESH_RATE").parse().unwrap_or(10));

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");

    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let rng = esp_hal::rng::Rng::new();

    let (mut wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let mut device = interfaces.sta;
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    // we can set a hostname here (or add other DHCP options)
    dhcp_socket.set_outgoing_options(&[DhcpOption {
        kind: 12,
        data: b"implRust",
    }]);
    socket_set.add(dhcp_socket);

    let now = || Instant::now().duration_since_epoch().as_millis();
    let stack = Stack::new(
        create_interface(&mut device),
        device,
        socket_set,
        now,
        rng.random(),
    );

    log::info!("Connecting to Wi-Fi...");

    connect_to_wifi(
        &mut wifi_controller,
        NetworkConfig {
            ssid: env!("WIFI_SSID"),
            password: env!("WIFI_PASSWORD"),
        },
    )
    .expect("Failed to connect to Wi-Fi");

    log::info!("Acquiring IP address via DHCP...");
    loop {
        stack.work();
        if stack.is_iface_up() {
            log::info!("IP acquired: {:?}", stack.get_ip_info());
            break;
        }
    }

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

    let mut rx_buffer = [0u8; 1124];
    let mut tx_buffer = [0u8; 1124];
    let server_address =
        smoltcp::wire::IpAddress::Ipv4(Ipv4Addr::from_str(env!("SERVER_ADDRESS")).unwrap());
    let server_port: u16 = env!("SERVER_PORT").parse().unwrap();

    let response = b"Ok";
    let number_of_pixels_in_bytes = (epd13in3::EPD_HEIGHT * epd13in3::EPD_WIDTH / 2) as usize;

    let mut socket = stack.get_socket(&mut rx_buffer, &mut tx_buffer);
    loop {
        socket.work();
        info!(
            "Opening socket to server {:?}:{:?}...",
            server_address, server_port
        );
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

        let mut buffer_read = 0;
        let mut buff_reader = [0u8; 1024];
        epd.select_left_panel();
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
                        log::info!("Finished reading left panel data.");
                        epd.send_data_bytes(&buff_reader[..n]);
                        epd.select_right_panel();
                    } else {
                        log::info!(
                            "Finished reading left panel data. Starting to read right panel data."
                        );
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
            if let Err(e) = socket.flush() {
                log::error!("Socket flush error: {:?}", e);
                break;
            }
        }
        log::info!("Finished reading from socket. Total bytes read: {buffer_read}");
        epd.turn_on_display();

        let delay_start = Instant::now();
        socket.close();

        let timer = TimerWakeupSource::new(refresh_rate);
        rtc.sleep_deep(&[&timer]);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples/src/bin
}
