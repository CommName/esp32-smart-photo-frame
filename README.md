# Smart Picture Frame - ESP32 E-Paper Display

A `no_std` Rust implementation of the Waveshare 13.3" e-Paper display driver for ESP32 using `esp-hal`. This project displays images on a 6-color e-ink display with 1200x1600 resolution.

## Features

- **No standard library (`no_std`)** - Minimal memory footprint
- **ESP-HAL based** - Direct hardware access, fast and efficient
- **6-color display** - Black, White, Yellow, Red, Blue, Green
- **Software SPI** - Bit-banging implementation for maximum compatibility
- **Multiple display modes** - Full screen, partial updates, clear
- **Static patterns** - Built-in test patterns that don't require heap allocation
- **Low power sleep mode** - Energy efficient operation

## Hardware Connections

| ESP32 GPIO | E-Paper Pin | Function |
|------------|-------------|----------|
| GPIO13     | SCK         | SPI Clock |
| GPIO14     | MOSI        | SPI Data Out |
| GPIO15     | CS_M        | Chip Select Master (Left Panel) |
| GPIO2      | CS_S        | Chip Select Slave (Right Panel) |
| GPIO26     | RST         | Reset |
| GPIO27     | DC          | Data/Command Select |
| GPIO25     | BUSY        | Busy Signal Input |
| GPIO33     | PWR         | Power Control |

## Prerequisites

1. **Install Rust toolchain:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install ESP32 toolchain:**
   ```bash
   cargo install espup
   espup install
   # On Linux/Mac:
   . $HOME/export-esp.sh
   # On Windows:
   # %USERPROFILE%\export-esp.ps1
   ```

3. **Install espflash:**
   ```bash
   cargo install espflash
   ```

## Building

Navigate to the project directory and build:

```bash
cd smart-picture-frame
cargo build --release
```

## Flashing to ESP32

### Automatic (recommended):
```bash
cargo run --release
```

### Manual:
```bash
espflash flash target/xtensa-esp32-none-elf/release/smart-picture-frame --monitor
```

## Project Structure

```
smart-picture-frame/
├── src/
│   ├── bin/
│   │   └── main.rs          # Main application entry point
│   ├── lib.rs               # Library exports
│   ├── dev_config.rs        # GPIO and hardware configuration
│   ├── epd_13in3e.rs        # E-Paper display driver
│   └── image_data.rs        # Image data and pattern generators
├── .cargo/
│   └── config.toml          # Cargo build configuration
├── Cargo.toml               # Project dependencies
├── build.rs                 # Build script
├── rust-toolchain.toml      # Rust toolchain specification
└── README.md                # This file
```

## Usage

The main application demonstrates three display modes:

### 1. Display Bitmap Image
```rust
epd.display_part(BMP_1, 400, 500, 400, 600);
```

### 2. Display Test Pattern
```rust
let test_pattern = generate_test_pattern_static();
epd.display(test_pattern);
```

### 3. Display Color Blocks
```rust
let color_blocks = generate_color_blocks_static();
epd.display(color_blocks);
```

## API Overview

### DevConfig
Hardware configuration struct managing GPIO pins and SPI communication.

**Methods:**
- `new(...) -> Self` - Initialize GPIO pins
- `delay_ms(ms: u32)` - Delay for specified milliseconds
- `spi_write_byte(data: u8)` - Write single byte via software SPI
- `spi_write_bytes(data: &[u8])` - Write multiple bytes
- `module_exit()` - Power down the display

### EPD13in3e
Main display driver.

**Methods:**
- `new(config: DevConfig) -> Self` - Create display instance
- `init(&mut self)` - Initialize display with configuration
- `clear(&mut self, color: Color)` - Clear entire screen
- `display(&mut self, image: &[u8])` - Display full-screen image
- `display_part(&mut self, image, x, y, width, height)` - Display partial image
- `sleep(&mut self)` - Enter low power mode
- `module_exit(&mut self)` - Complete shutdown

### Color Enum
```rust
pub enum Color {
    Black,   // 0x0
    White,   // 0x1
    Yellow,  // 0x2
    Red,     // 0x3
    Blue,    // 0x5
    Green,   // 0x6
}
```

## Image Format

Images are encoded with 4-bit color values (2 pixels per byte):
- Each byte contains two pixels: `[pixel1 (high nibble)][pixel2 (low nibble)]`
- For a 1200x1600 display, you need 600x1600 = 960,000 bytes
- For partial images, calculate accordingly

Example:
```rust
// Two white pixels
0x11

// White pixel followed by red pixel
0x13

// Color block pattern
let byte = (color1 << 4) | color2;
```

## Adding Custom Images

### Option 1: Include Binary Data
```rust
// In src/image_data.rs
pub const MY_IMAGE: &[u8] = include_bytes!("../images/my_image.bin");
```

### Option 2: Define Array
```rust
pub const MY_IMAGE: &[u8] = &[
    0x11, 0x22, 0x33, // ... your image data
];
```

### Option 3: Generate Patterns
```rust
pub fn my_pattern() -> &'static [u8] {
    static PATTERN: [u8; SIZE] = {
        // Generate pattern at compile time
        // ...
    };
    &PATTERN
}
```

## Memory Considerations

This is a `no_std` project, so:
- **No heap allocation** - All buffers are stack-allocated or static
- **Fixed-size buffers** - Maximum buffer size is 300 bytes for display operations
- **Static patterns** - Test patterns are generated at compile time
- **Efficient SPI** - Bit-banging minimizes memory usage

## Differences from `picture` Project

| Feature | `picture` (esp-idf-hal) | `smart-picture-frame` (esp-hal) |
|---------|------------------------|----------------------------------|
| Standard Library | `std` | `no_std` |
| HAL | esp-idf-hal | esp-hal |
| Memory Allocation | Heap (Vec) | Stack/Static only |
| Logging | `log` crate | None (no_std) |
| Error Handling | `anyhow::Result` | Direct error handling |
| Size | Larger binary | Smaller binary |
| Boot Time | Slower (OS overhead) | Faster (bare metal) |

## Troubleshooting

### Build Errors

**Error: `espup` not found**
```bash
cargo install espup
espup install
```

**Error: linker not found**
- Make sure you've sourced the export script: `. $HOME/export-esp.sh`

### Flash Errors

**Cannot open serial port**
- Check USB connection
- Ensure no other program is using the serial port
- Try: `espflash flash --port /dev/ttyUSB0 ...`

**Flash fails**
- Hold BOOT button while flashing
- Reduce flash speed: `espflash flash --speed 115200 ...`

### Display Issues

**Display not responding**
- Verify all GPIO connections match the pin configuration
- Check 3.3V power supply is adequate (display draws significant current)
- Ensure BUSY pin is properly connected

**Partial display not working**
- Check x/y coordinates are within valid range
- Ensure image data size matches specified dimensions

## Performance

- **Initialization:** ~2-3 seconds
- **Clear screen:** ~5-10 seconds (depends on color)
- **Full display update:** ~10-15 seconds
- **Partial update:** ~3-5 seconds (depends on size)

## License

MIT License - Same as original Waveshare code

## Credits

- **Original C++ implementation:** Waveshare team
- **Rust port:** Adapted for ESP32 with esp-hal
- **Framework:** ESP-RS community

## Further Development

Ideas for enhancement:
- Add Wi-Fi support for remote image updates
- Implement image conversion utilities
- Add support for more image formats
- Create web interface for image selection
- Battery management for portable operation

## Resources

- [ESP-HAL Documentation](https://docs.esp-rs.org/esp-hal/)
- [Waveshare 13.3" E-Paper Specs](https://www.waveshare.com/13.3inch-e-paper-hat.htm)
- [ESP32 Datasheet](https://www.espressif.com/en/products/socs/esp32)
