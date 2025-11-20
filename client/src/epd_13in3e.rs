use log::info;

use crate::dev_config::DevConfig;

// Display dimensions
pub const EPD_WIDTH: u16 = 1200;
pub const EPD_HEIGHT: u16 = 1600;

// Color definitions
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    White = 0x1,
    Yellow = 0x2,
    Red = 0x3,
    Blue = 0x5,
    Green = 0x6, // ORANGE
}

pub enum Command {
    PSR = 0x00,
    PWR_EPD = 0x01,
    POF = 0x02,
    PON = 0x04,
    BTST_N = 0x05,
    BTST_P = 0x06,
    DTM = 0x10,
    DRF = 0x12,
    CDI = 0x50,
    TCON = 0x60,
    TRES = 0x61,
    AN_TM = 0x74,
    AGID = 0x86,
    BUCK_BOOST_VDDN = 0xB0,
    TFT_VCOM_POWER = 0xB1,
    EN_BUF = 0xB6,
    BOOST_VDDP_EN = 0xB7,
    CCSET = 0xE0,
    PWS = 0xE3,
    CMD66 = 0xF0,
}

// Register values
const PSR_V: [u8; 2] = [0xDF, 0x69];
const PWR_V: [u8; 6] = [0x0F, 0x00, 0x28, 0x2C, 0x28, 0x38];
const POF_V: [u8; 1] = [0x00];
const DRF_V: [u8; 1] = [0x00];
const CDI_V: [u8; 1] = [0xF7];
const TCON_V: [u8; 2] = [0x03, 0x03];
const TRES_V: [u8; 4] = [0x04, 0xB0, 0x03, 0x20];
const CMD66_V: [u8; 6] = [0x49, 0x55, 0x13, 0x5D, 0x05, 0x10];
const EN_BUF_V: [u8; 1] = [0x07];
const CCSET_V: [u8; 1] = [0x01];
const PWS_V: [u8; 1] = [0x22];
const AN_TM_V: [u8; 9] = [0xC0, 0x1C, 0x1C, 0xCC, 0xCC, 0xCC, 0x15, 0x15, 0x55];
const AGID_V: [u8; 1] = [0x10];
const BTST_P_V: [u8; 2] = [0xE8, 0x28];
const BOOST_VDDP_EN_V: [u8; 1] = [0x01];
const BTST_N_V: [u8; 2] = [0xE8, 0x28];
const BUCK_BOOST_VDDN_V: [u8; 1] = [0x01];
const TFT_VCOM_POWER_V: [u8; 1] = [0x02];

pub struct EPD13in3e {
    config: DevConfig,
}

impl EPD13in3e {
    pub fn new(config: DevConfig) -> Self {
        EPD13in3e { config }
    }

    pub fn cs_all(&mut self, high: bool) {
        if high {
            self.config.cs_m.set_high();
            self.config.cs_s.set_high();
        } else {
            self.config.cs_m.set_low();
            self.config.cs_s.set_low();
        }
    }

    fn reset(&mut self) {
        self.config.rst.set_high();
        self.config.delay_ms(30);
        self.config.rst.set_low();
        self.config.delay_ms(30);
        self.config.rst.set_high();
        self.config.delay_ms(30);
        self.config.rst.set_low();
        self.config.delay_ms(30);
        self.config.rst.set_high();
        self.config.delay_ms(30);
    }

    fn send_command(&mut self, cmd: Command) {
        self.config.dc.set_low();
        self.config.spi_write_byte(cmd as u8);
    }

    fn send_data(&mut self, data: u8) {
        self.config.dc.set_high();
        self.config.spi_write_byte(data);
    }

    pub fn send_data_bytes(&mut self, data: &[u8]) {
        self.config.dc.set_high();
        self.config.spi_write_bytes(data);
    }

    fn spi_send(&mut self, cmd: Command, data: &[u8]) {
        self.cs_all(false);
        self.send_command(cmd);
        self.send_data_bytes(data);
        self.cs_all(true);
    }

    pub fn turn_on_display(&mut self) {
        self.cs_all(true);
        self.cs_all(false);
        self.send_command(Command::PON);
        self.cs_all(true);
        self.wait_upon_idle_high();

        self.config.delay_ms(50);
        self.spi_send(Command::DRF, &[0x00]);
        self.wait_upon_idle_high();
        self.wait_upon_idle_high();

        self.spi_send(Command::POF, &[0x00]);
        self.wait_upon_idle_high();

        // deep sleep
        self.spi_send(Command::EN_BUF, &[0xA5]);
    }

    fn wait_upon_idle_high(&mut self) {
        while self.config.busy.is_low() {
            self.config.delay_ms(100);
        }
        self.config.delay_ms(100);
    }

    pub fn init(&mut self) {
        self.reset();
        info!("EPD Reset complete");
        self.wait_upon_idle_high();
        info!("EPD Idle high complete");

        self.spi_send(Command::AN_TM, &AN_TM_V);
        self.spi_send(Command::CMD66, &CMD66_V);
        self.spi_send(Command::PSR, &PSR_V);
        self.spi_send(Command::CDI, &CDI_V);
        self.spi_send(Command::TCON, &TCON_V);
        self.spi_send(Command::AGID, &AGID_V);
        self.spi_send(Command::PWS, &PWS_V);
        self.spi_send(Command::CCSET, &CCSET_V);
        self.spi_send(Command::TRES, &TRES_V);

        self.config.cs_m.set_low();
        self.send_command(Command::PWR_EPD);
        self.send_data_bytes(&PWR_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.send_command(Command::EN_BUF);
        self.send_data_bytes(&EN_BUF_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.send_command(Command::BTST_P);
        self.send_data_bytes(&BTST_P_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.send_command(Command::BOOST_VDDP_EN);
        self.send_data_bytes(&BOOST_VDDP_EN_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.send_command(Command::BTST_N);
        self.send_data_bytes(&BTST_N_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.send_command(Command::BUCK_BOOST_VDDN);
        self.send_data_bytes(&BUCK_BOOST_VDDN_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.send_command(Command::TFT_VCOM_POWER);
        self.send_data_bytes(&TFT_VCOM_POWER_V);
        self.cs_all(true);
    }

    pub fn clear(&mut self, color: Color) {
        let width = EPD_WIDTH / 4;

        let color_byte = (color as u8) << 4 | (color as u8);
        let buf_size = (width / 2) as usize;

        // Create buffer on stack to avoid heap allocation in no_std
        let mut buf = [0u8; 300]; // width/2 max size
        for i in 0..width {
            buf[i as usize] = color_byte;
        }

        self.config.cs_m.set_low();
        self.send_command(Command::DTM);
        for _ in 0..EPD_HEIGHT {
            self.send_data_bytes(&buf);
            self.config.delay_ms(1);
        }
        self.cs_all(true);

        self.config.cs_s.set_low();
        self.send_command(Command::DTM);
        for _ in 0..EPD_HEIGHT {
            self.send_data_bytes(&buf[..buf_size.min(600)]);
            self.config.delay_ms(1);
        }
        self.cs_all(true);

        self.turn_on_display();
    }

    pub fn set_left_panel(&mut self) {
        self.cs_all(true);
        self.config.cs_m.set_low();
        self.send_command(Command::DTM);
    }

    pub fn set_right_panel(&mut self) {
        self.cs_all(true);
        self.config.cs_s.set_low();
        self.send_command(Command::DTM);
    }

    pub fn display(&mut self, image: &[u8]) {
        let width = if EPD_WIDTH % 2 == 0 {
            EPD_WIDTH / 2
        } else {
            EPD_WIDTH / 2 + 1
        };
        let width1 = if width % 2 == 0 {
            width / 2
        } else {
            width / 2 + 1
        } as usize;

        self.config.cs_m.set_low();
        self.send_command(Command::DTM);
        for i in 0..EPD_HEIGHT as usize {
            let start = i * width as usize;
            let end = start + width1;
            if end <= image.len() {
                self.send_data_bytes(&image[start..end]);
            }
            self.config.delay_ms(1);
        }
        self.cs_all(true);

        self.config.cs_s.set_low();
        self.send_command(Command::DTM);
        for i in 0..EPD_HEIGHT as usize {
            let start = i * width as usize + width1;
            let end = start + width1;
            if end <= image.len() {
                self.send_data_bytes(&image[start..end]);
            }
            self.config.delay_ms(1);
        }
        self.cs_all(true);

        self.turn_on_display();
    }

    pub fn sleep(&mut self) {
        self.cs_all(false);
        self.send_command(Command::POF);
        self.send_data(0x00);
        self.cs_all(true);
        self.config.delay_ms(2000);
    }

    pub fn module_exit(&mut self) {
        self.config.module_exit();
    }
}
