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

    fn send_data_bytes(&mut self, data: &[u8]) {
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

    pub fn clear(&mut self, color: Color) {
        let width = if EPD_WIDTH % 2 == 0 {
            EPD_WIDTH / 2
        } else {
            EPD_WIDTH / 2 + 1
        };

        let color_byte = (color as u8) << 4 | (color as u8);
        let _buf_size = (width / 2) as usize;

        // Create buffer on stack to avoid heap allocation in no_std
        let mut buf = [0u8; 600]; // width/2 max size
        for i in 0..600 {
            buf[i] = color_byte;
        }

        self.config.cs_m.set_low();
        self.send_command(DTM);
        for _ in 0..EPD_HEIGHT {
            self.send_data_bytes(&buf);
            self.config.delay_ms(1);
        }
        self.cs_all(true);

        self.config.cs_s.set_low();
        self.send_command(DTM);
        for _ in 0..EPD_HEIGHT {
            self.send_data_bytes(&buf);
            self.config.delay_ms(1);
        }
        self.cs_all(true);

        self.turn_on_display();
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
        self.send_command(DTM);
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
        self.send_command(DTM);
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

    pub fn display_part(
        &mut self,
        image: &[u8],
        mut xstart: u16,
        ystart: u16,
        img_width: u16,
        img_height: u16,
    ) {
        let width = 600;
        let width1 = 300;
        let height = 1600;

        let mut xend = if (xstart + img_width) % 2 == 0 {
            (xstart + img_width) / 2
        } else {
            (xstart + img_width) / 2 + 1
        };
        let yend = ystart + img_height;
        xstart = xstart / 2;

        // Three cases: right panel only, left panel only, or spanning both panels
        if xstart > 300 {
            // Image is entirely on the right panel (CS_S)
            xend = xend - 300;
            xstart = xstart - 300;

            // Left panel (CS_M) - fill with white
            self.config.cs_m.set_low();
            self.send_command(DTM);
            for _ in 0..height {
                for _ in 0..width1 {
                    self.send_data(0x11);
                }
                self.config.delay_ms(1);
            }
            self.cs_all(true);

            // Right panel (CS_S) - display the image
            self.config.cs_s.set_low();
            self.send_command(DTM);
            for i in 0..height {
                for j in 0..width1 {
                    if i < yend && i >= ystart && j < xend && j >= xstart {
                        let img_idx = ((j - xstart) + (img_width / 2 * (i - ystart))) as usize;
                        if img_idx < image.len() {
                            self.send_data(image[img_idx]);
                        } else {
                            self.send_data(0x11);
                        }
                    } else {
                        self.send_data(0x11);
                    }
                }
                self.config.delay_ms(1);
            }
            self.cs_all(true);
        } else if xend < 300 {
            // Image is entirely on the left panel (CS_M)
            self.config.cs_m.set_low();
            self.send_command(DTM);
            for i in 0..height {
                for j in 0..width1 {
                    if i < yend && i >= ystart && j < xend && j >= xstart {
                        let img_idx = ((j - xstart) + (img_width / 2 * (i - ystart))) as usize;
                        if img_idx < image.len() {
                            self.send_data(image[img_idx]);
                        } else {
                            self.send_data(0x11);
                        }
                    } else {
                        self.send_data(0x11);
                    }
                }
                self.config.delay_ms(1);
            }
            self.cs_all(true);

            // Right panel (CS_S) - fill with white
            self.config.cs_s.set_low();
            self.send_command(DTM);
            for _ in 0..height {
                for _ in 0..width1 {
                    self.send_data(0x11);
                }
                self.config.delay_ms(1);
            }
            self.cs_all(true);
        } else {
            // Image spans both panels
            // Left panel (CS_M)
            self.config.cs_m.set_low();
            self.send_command(DTM);
            for i in 0..height {
                for j in 0..width1 {
                    if i < yend && i >= ystart && j >= xstart {
                        let img_idx = ((j - xstart) + (img_width / 2 * (i - ystart))) as usize;
                        if img_idx < image.len() {
                            self.send_data(image[img_idx]);
                        } else {
                            self.send_data(0x11);
                        }
                    } else {
                        self.send_data(0x11);
                    }
                }
                self.config.delay_ms(1);
            }
            self.cs_all(true);

            // Right panel (CS_S)
            self.config.cs_s.set_low();
            self.send_command(DTM);
            for i in 0..height {
                for j in 0..width1 {
                    if i < yend && i >= ystart && j < (xend - 300) {
                        let img_idx =
                            ((j + 300 - xstart) + (img_width / 2 * (i - ystart))) as usize;
                        if img_idx < image.len() {
                            self.send_data(image[img_idx]);
                        } else {
                            self.send_data(0x11);
                        }
                    } else {
                        self.send_data(0x11);
                    }
                }
                self.config.delay_ms(1);
            }
            self.cs_all(true);
        }

        self.turn_on_display();
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
