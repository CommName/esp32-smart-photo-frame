use esp_println::println;

use crate::dev_config::DevConfig;

// Display dimensions
pub const EPD_WIDTH: usize = 1200;
pub const EPD_HEIGHT: usize = 1600;

// Color definitions
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    White = 0x1,
    Yellow = 0x2,
    Red = 0x3,
    Blue = 0x5,
    Green = 0x6,
}

// Command definitions
const PSR: u8 = 0x00;
const PWR_EPD: u8 = 0x01;
const POF: u8 = 0x02;
const PON: u8 = 0x04;
const BTST_N: u8 = 0x05;
const BTST_P: u8 = 0x06;
const DTM: u8 = 0x10;
const DRF: u8 = 0x12;
const CDI: u8 = 0x50;
const TCON: u8 = 0x60;
const TRES: u8 = 0x61;
const AN_TM: u8 = 0x74;
const AGID: u8 = 0x86;
const BUCK_BOOST_VDDN: u8 = 0xB0;
const TFT_VCOM_POWER: u8 = 0xB1;
const EN_BUF: u8 = 0xB6;
const BOOST_VDDP_EN: u8 = 0xB7;
const CCSET: u8 = 0xE0;
const PWS: u8 = 0xE3;
const CMD66: u8 = 0xF0;

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

    fn cs_all(&mut self, high: bool) {
        if high {
            self.config.cs_m.set_high();
            self.config.cs_s.set_high();
        } else {
            self.config.cs_m.set_low();
            self.config.cs_s.set_low();
        }
    }

    // Software reset
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

    fn send_command(&mut self, cmd: u8) {
        //self.config.dc.set_low();
        self.config.spi_write_byte(cmd);
    }

    fn send_data(&mut self, data: u8) {
        //self.config.dc.set_high();
        self.config.spi_write_byte(data);
    }

    pub fn send_data_bytes(&mut self, data: &[u8]) {
        //self.config.dc.set_high();
        self.config.spi_write_bytes(data);
    }

    fn spi_send(&mut self, cmd: u8, data: &[u8]) {
        self.send_command(cmd);
        self.send_data_bytes(data);
    }

    fn read_busy(&mut self) {
        while self.config.busy.is_low() {
            self.config.delay_ms(10);
        }
        self.config.delay_ms(20);
    }

    pub fn turn_on_display(&mut self) {
        self.cs_all(true);
        self.cs_all(false);
        self.send_command(PON);
        self.cs_all(true);
        self.read_busy();

        self.config.delay_ms(50);
        self.cs_all(false);
        self.spi_send(DRF, &DRF_V);
        self.cs_all(true);
        self.read_busy();

        self.cs_all(false);
        self.spi_send(POF, &POF_V);
        self.cs_all(true);
    }

    pub fn init(&mut self) {
        self.reset();

        self.config.cs_m.set_low();
        self.spi_send(AN_TM, &AN_TM_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(CMD66, &CMD66_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(PSR, &PSR_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(CDI, &CDI_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(TCON, &TCON_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(AGID, &AGID_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(PWS, &PWS_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(CCSET, &CCSET_V);
        self.cs_all(true);

        self.cs_all(false);
        self.spi_send(TRES, &TRES_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(PWR_EPD, &PWR_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(EN_BUF, &EN_BUF_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(BTST_P, &BTST_P_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(BOOST_VDDP_EN, &BOOST_VDDP_EN_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(BTST_N, &BTST_N_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(BUCK_BOOST_VDDN, &BUCK_BOOST_VDDN_V);
        self.cs_all(true);

        self.config.cs_m.set_low();
        self.spi_send(TFT_VCOM_POWER, &TFT_VCOM_POWER_V);
        self.cs_all(true);
    }

    pub fn send_byte(&mut self, data: u8) {
        self.send_data(data);
    }

    pub fn select_left_panel(&mut self) {
        println!("Selecting left panel...");
        self.cs_all(true);
        self.config.cs_m.set_low();
        self.config.cs_s.set_high();
        self.send_command(DTM);
    }

    pub fn select_right_panel(&mut self) {
        println!("Selecting right panel...");
        self.cs_all(true);
        self.config.cs_m.set_high();
        self.config.cs_s.set_low();
        self.send_command(DTM);
    }

    pub fn sleep(&mut self) {
        self.cs_all(false);
        self.send_command(POF);
        self.send_data(0x00);
        self.cs_all(true);
        self.config.delay_ms(2000);
    }

    pub fn module_exit(&mut self) {
        self.config.module_exit();
    }
}
