use esp_hal::gpio::{Input, Output};

/// GPIO Pin Configuration for ESP32 E-Paper Display
pub struct DevConfig {
    pub sck: Output<'static>,
    pub mosi: Output<'static>,
    pub cs_m: Output<'static>,
    pub cs_s: Output<'static>,
    pub rst: Output<'static>,
    pub dc: Output<'static>,
    pub busy: Input<'static>,
    pub pwr: Output<'static>,
}

impl DevConfig {
    pub fn new(
        sck: Output<'static>,
        mosi: Output<'static>,
        mut cs_m: Output<'static>,
        mut cs_s: Output<'static>,
        rst: Output<'static>,
        dc: Output<'static>,
        busy: Input<'static>,
        mut pwr: Output<'static>,
    ) -> Self {
        // Initialize pins
        cs_m.set_high();
        cs_s.set_high();
        pwr.set_high();

        DevConfig {
            sck,
            mosi,
            cs_m,
            cs_s,
            rst,
            dc,
            busy,
            pwr,
        }
    }

    pub fn delay_ms(&self, ms: u32) {
        esp_hal::time::Instant::now()
            .elapsed()
            .checked_add(esp_hal::time::Duration::from_millis(ms as u64));
        let start = esp_hal::time::Instant::now();
        while start.elapsed() < esp_hal::time::Duration::from_millis(ms as u64) {}
    }

    pub fn spi_write_byte(&mut self, data: u8) {
        for i in (0..8).rev() {
            if (data & (1 << i)) != 0 {
                self.mosi.set_high();
            } else {
                self.mosi.set_low();
            }

            self.sck.set_high();
            self.sck.set_low();
        }
    }

    pub fn spi_write_bytes(&mut self, data: &[u8]) {
        for byte in data {
            self.spi_write_byte(*byte);
        }
    }

    pub fn module_exit(&mut self) {
        self.pwr.set_low();
        self.rst.set_low();
    }
}
