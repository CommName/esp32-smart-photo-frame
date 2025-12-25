use core::time::Duration;

use super::dev_config::DevConfig;
use super::epd_13in3e::{EPD_HEIGHT, EPD_WIDTH, EPD13in3e};
use super::wifi::*;
use blocking_network_stack::{Socket, Stack};
use embedded_io::Read;
use embedded_io::Write;
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::rtc_cntl::sleep::TimerWakeupSource;
use esp_hal::time::{self};
use esp_println::println;
use esp_radio::wifi::Interfaces;
use log::{debug, info};
use smoltcp::iface::{SocketSet, SocketStorage};
use smoltcp::wire::IpAddress;

fn setup_network<'a>(
    wifi_controller: &'a mut esp_radio::wifi::WifiController,
    socket_set: SocketSet<'a>,
    interfaces: Interfaces<'a>,
    ssid: &str,
    password: &str,
) -> Stack<'a, esp_radio::wifi::WifiDevice<'a>> {
    info!("Connecting to Wi-Fi network");
    connect_wifi(wifi_controller, ssid, password);
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

    info!("Initializing RNG and network stack");
    let rng = Rng::new();
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let mut stack: Stack<'a, WifiDevice<'a>> =
        Stack::new(iface, device, socket_set, now, rng.random());
    info!("Network stack initialized");

    stack
        .set_iface_configuration(&blocking_network_stack::ipv4::Configuration::Client(
            blocking_network_stack::ipv4::ClientConfiguration::DHCP(Default::default()),
        ))
        .expect("Failed to set interface configuration");

    stack
}

fn receive_image<'a>(
    epd: &mut EPD13in3e,
    socket: &mut Socket<'a, 'a, esp_radio::wifi::WifiDevice<'a>>,
    destination_ip: IpAddress,
    destination_port: u16,
) {
    info!("Allocating buffers - RX: 1536 bytes, TX: 1536 bytes, Image: 240000 bytes");
    // Initialize and clear display
    // Create EPD instance
    info!("Initializing e-paper display");
    epd.init();

    info!("Creating socket from network stack");
    socket.work();
    info!("Socket created and initialized");

    socket.open(destination_ip, destination_port).unwrap();
    info!("Connection established");

    info!("Starting to receive image data");
    let mut bytes_read = 0;
    let mut left_size = true;
    epd.cs_all(true);
    epd.set_left_panel();

    if let Err(e) = socket.write(b"OK") {
        log::error!("Failed to send data: {e:?}");
    }
    if let Err(e) = socket.flush() {
        log::error!("Failed to flush data: {e:?}");
    }

    loop {
        socket.work();
        let mut response_buffer = [0u8; 1024];
        let response = match socket.read(&mut response_buffer) {
            Ok(size) => size,
            Err(e) => {
                log::error!("Failed to read data: {e:?}");
                break;
            }
        };

        if let Err(e) = socket.write(b"OK") {
            log::error!("Failed to send data: {e:?}");
            break;
        }
        if let Err(e) = socket.flush() {
            log::error!("Failed to flush data: {e:?}");
            break;
        }

        debug!("Received {} bytes from server", response);

        let limit = EPD_WIDTH as usize * EPD_HEIGHT as usize / 4;
        let offset_limit: i32 = limit as i32 - bytes_read as i32 - response as i32;
        // info!("Bytes remaining to fill left panel: {}", offsetLimit);
        if left_size && offset_limit < 0 {
            println!(
                "Offset limit exceeded, switching to right panel {offset_limit}, total bytes read: {bytes_read}, response bytes: {response}"
            );
            epd.send_data_bytes(&response_buffer[..(limit - bytes_read)]);

            info!("Left panel image received switching to left panel");

            // epd.turn_on_display();

            epd.set_right_panel();
            epd.send_data_bytes(&response_buffer[(limit - bytes_read)..response].as_ref());
            bytes_read += response;
            left_size = false;
        } else if left_size && offset_limit == 0 as i32 {
            println!("Exact fit for left panel received");
            epd.send_data_bytes(&response_buffer[..response]);
            epd.set_right_panel();
            left_size = false;
            bytes_read += response;
        } else {
            epd.send_data_bytes(&response_buffer[..response]);
            bytes_read += response;
        }

        debug!("Total image data received: {} bytes", bytes_read);

        if bytes_read == 960000 {
            break;
        }
    }

    info!("Total image data received: {} bytes", bytes_read);

    epd.cs_all(true);
    epd.turn_on_display();
}

fn enter_sleep_mode(rtc: &mut Rtc) {
    // 30 minutes in microseconds
    const SLEEP_TIME_US: u64 = 30 * 60 * 1_000_000;
    let timer = TimerWakeupSource::new(Duration::from_secs(SLEEP_TIME_US));
    rtc.sleep_deep(&[&timer]);
}

pub fn run(
    dev_config: DevConfig,
    mut wifi_controller: esp_radio::wifi::WifiController,
    interface: Interfaces,
    ssid: &str,
    destion_ip_address: IpAddress,
    destion_port: u16,
    password: &str,
    rtc: &mut Rtc,
) -> ! {
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let stack = setup_network(&mut wifi_controller, socket_set, interface, ssid, password);
    let mut epd = EPD13in3e::new(dev_config);

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut socket: Socket<'_, '_, WifiDevice<'_>> =
        stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {
        // Initialize device configuration with GPIO pins
        info!("E-paper display module initialized");
        receive_image(&mut epd, &mut socket, destion_ip_address, destion_port);

        info!("Enter sleep mode");
        enter_sleep_mode(rtc);
    }
}
