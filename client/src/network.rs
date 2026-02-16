use blocking_network_stack::Stack;
use esp_hal::time::Instant;
use esp_radio::wifi::{self, ClientConfig, WifiController};
use smoltcp::{
    iface::{SocketSet, SocketStorage},
    wire::DhcpOption,
};

pub struct NetworkConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

pub fn connect_to_wifi(
    wifi_controler: &mut WifiController,
    credentials: NetworkConfig,
) -> Result<(), &'static str> {
    // Placeholder implementation
    wifi_controler
        .set_config(&wifi::ModeConfig::Client(
            ClientConfig::default()
                .with_ssid(credentials.ssid.into())
                .with_auth_method(wifi::AuthMethod::Wpa2Personal)
                .with_password(credentials.password.into()),
        ))
        .map_err(|_| "Failed to setup wifi")?;

    wifi_controler
        .start()
        .map_err(|_| "Failed to start Wi-Fi controller")?;
    wifi_controler
        .connect()
        .map_err(|_| "Failed to connect to Wi-Fi network")?;

    while !wifi_controler
        .is_connected()
        .map_err(|_| "Failed to get connection status")?
    {
        // Wait until connected
    }
    log::info!("Connected to Wi-Fi network: {}", credentials.ssid);

    Ok(())
}

pub fn create_interface(device: &mut esp_radio::wifi::WifiDevice) -> smoltcp::iface::Interface {
    // users could create multiple instances but since they only have one WifiDevice
    // they probably can't do anything bad with that
    smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&device.mac_address()),
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
