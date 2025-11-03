/// Sample image data for 400x600 display (6 color)
/// This is a subset of the image data - for full image, include the complete array
/// Each byte encodes 2 pixels in 4-bit format
pub const BMP_1: &[u8] = &[
    0x33, 0x23, 0x00, 0x00, 0x00, 0x00, 0x33, 0x33, 0x33, 0x33, 0x30, 0x32, 0x32, 0x36, 0x32, 0x32,
    0x32, 0x32, 0x32, 0x32, 0x33, 0x32, 0x33, 0x32, 0x33, 0x23, 0x23, 0x23, 0x23, 0x22, 0x23, 0x23,
    0x22, 0x22, 0x23, 0x22, 0x32, 0x22, 0x23, 0x23, 0x23, 0x12, 0x11, 0x23, 0x31, 0x11, 0x22, 0x33,
    0x21, 0x21, 0x13, 0x21, 0x13, 0x21, 0x21, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x13, 0x21,
    0x12,
    // ... Add more data as needed
    // For full implementation, you would include all 120000 bytes from the original ImageData.cpp
];

/// Generate a simple test pattern for demonstration (no_std compatible)
/// Returns a static buffer with the pattern
pub fn generate_test_pattern_static() -> &'static [u8] {
    // In no_std, we can't allocate dynamically, so we use a static pattern
    static PATTERN: [u8; 1000] = {
        let mut pattern = [0u8; 1000];
        let mut i = 0;
        while i < 1000 {
            pattern[i] = ((i % 6) as u8) << 4 | ((i / 100) % 6) as u8;
            i += 1;
        }
        pattern
    };
    &PATTERN
}

/// Generate color blocks for testing (static version for no_std)
pub fn generate_color_blocks_static() -> &'static [u8] {
    // Creates a repeating pattern of 6 color blocks
    static COLOR_BLOCKS: [u8; 600] = {
        let mut blocks = [0u8; 600];
        let mut i = 0;
        while i < 600 {
            let block_idx = (i / 100) % 6;
            blocks[i] = (block_idx << 4 | block_idx) as u8;
            i += 1;
        }
        blocks
    };
    &COLOR_BLOCKS
}
