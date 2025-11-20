pub use esp_radio::wifi::*;
use log::info;

pub fn connect_wifi(controller: &mut esp_radio::wifi::WifiController, ssid: &str, password: &str) {
    info!("Setting up Wi-Fi client configuration for SSID: {}", ssid);
    let mode_config = ModeConfig::Client(
        ClientConfig::default()
            .with_ssid(ssid.into())
            .with_password(password.into()),
    );

    info!("Applying Wi-Fi configuration");
    controller
        .set_config(&mode_config)
        .expect("Failed to set Wi-Fi configuration");

    info!("Starting Wi-Fi controller");
    controller
        .start()
        .expect("Failed to start Wi-Fi controller");

    info!("Initiating connection to Wi-Fi network");
    controller
        .connect()
        .expect("Failed to connect to Wi-Fi network");

    info!("Waiting for Wi-Fi connection to establish...");
    while !controller.is_connected().unwrap_or(false) {
        // Wait until connected
    }
    info!("Successfully connected to Wi-Fi network: {}", ssid);
}

pub fn create_interface(device: &mut esp_radio::wifi::WifiDevice) -> smoltcp::iface::Interface {
    // users could create multiple instances but since they only have one WifiDevice
    // they probably can't do anything bad with that
    let mac_address = device.mac_address();
    info!(
        "Creating network interface with MAC address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac_address[0],
        mac_address[1],
        mac_address[2],
        mac_address[3],
        mac_address[4],
        mac_address[5]
    );

    smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&mac_address),
        )),
        device,
        timestamp(),
    )
}

// some smoltcp boilerplate
fn timestamp() -> smoltcp::time::Instant {
    smoltcp::time::Instant::from_micros(
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as i64,
    )
}
pub fn parse_ip(ip: &str) -> [u8; 4] {
    info!("Parsing IP address: {}", ip);
    let mut result = [0u8; 4];
    for (idx, octet) in ip.split(".").into_iter().enumerate() {
        result[idx] = u8::from_str_radix(octet, 10).unwrap();
    }
    info!(
        "Parsed IP: {}.{}.{}.{}",
        result[0], result[1], result[2], result[3]
    );
    result
}
