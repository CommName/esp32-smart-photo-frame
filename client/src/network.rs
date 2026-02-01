use blocking_network_stack::Stack;
use esp_hal::time::Instant;
use esp_radio::wifi::{self, ClientConfig, WifiController};
use esp_rtos::TimerSource;
use smoltcp::{
    iface::{SocketSet, SocketStorage},
    socket::dhcpv4::Socket,
    wire::DhcpOption,
};

pub struct NetworkConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

pub fn create_stack<'a>(
    timer0: impl TimerSource,
    wifi: esp_hal::peripherals::WIFI<'a>,
    socket_set_entries: &'a mut [SocketStorage<'a>; 3],
    radio_init: &'a esp_radio::Controller<'a>,
) -> Stack<'a, wifi::WifiDevice<'a>> {
    let rng = esp_hal::rng::Rng::new();

    esp_rtos::start(timer0);
    let (mut wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, wifi, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let mut device = interfaces.sta;
    let mut socket_set: SocketSet<'a> = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    // we can set a hostname here (or add other DHCP options)
    dhcp_socket.set_outgoing_options(&[DhcpOption {
        kind: 12,
        data: b"implRust",
    }]);
    socket_set.add(dhcp_socket);

    let now = || Instant::now().duration_since_epoch().as_millis();
    let stack: Stack<'a, wifi::WifiDevice<'a>> = Stack::new(
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

    stack
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
