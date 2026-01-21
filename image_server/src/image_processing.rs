use image::{ImageBuffer, Rgb};

pub fn dither_with_palette(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, palette: &[Rgb<u8>]) {
    let (width, height) = img.dimensions();
    let mut error_buffer = vec![vec![(0.0, 0.0, 0.0); width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let old_r = pixel[0] as f32 + error_buffer[y as usize][x as usize].0;
            let old_g = pixel[1] as f32 + error_buffer[y as usize][x as usize].1;
            let old_b = pixel[2] as f32 + error_buffer[y as usize][x as usize].2;

            let closest_color = find_closest_color(old_r, old_g, old_b, palette);
            img.put_pixel(x, y, closest_color);

            let error_r = old_r - closest_color[0] as f32;
            let error_g = old_g - closest_color[1] as f32;
            let error_b = old_b - closest_color[2] as f32;

            // Distribute error to neighbors (Floyd-Steinberg coefficients)
            if x + 1 < width {
                error_buffer[y as usize][(x + 1) as usize].0 += error_r * 7.0 / 16.0;
                error_buffer[y as usize][(x + 1) as usize].1 += error_g * 7.0 / 16.0;
                error_buffer[y as usize][(x + 1) as usize].2 += error_b * 7.0 / 16.0;
            }
            if x > 0 && y + 1 < height {
                error_buffer[(y + 1) as usize][(x - 1) as usize].0 += error_r * 3.0 / 16.0;
                error_buffer[(y + 1) as usize][(x - 1) as usize].1 += error_g * 3.0 / 16.0;
                error_buffer[(y + 1) as usize][(x - 1) as usize].2 += error_b * 3.0 / 16.0;
            }
            if y + 1 < height {
                error_buffer[(y + 1) as usize][x as usize].0 += error_r * 5.0 / 16.0;
                error_buffer[(y + 1) as usize][x as usize].1 += error_g * 5.0 / 16.0;
                error_buffer[(y + 1) as usize][x as usize].2 += error_b * 5.0 / 16.0;
            }
            if x + 1 < width && y + 1 < height {
                error_buffer[(y + 1) as usize][(x + 1) as usize].0 += error_r * 1.0 / 16.0;
                error_buffer[(y + 1) as usize][(x + 1) as usize].1 += error_g * 1.0 / 16.0;
                error_buffer[(y + 1) as usize][(x + 1) as usize].2 += error_b * 1.0 / 16.0;
            }
        }
    }
}

fn find_closest_color(r: f32, g: f32, b: f32, palette: &[Rgb<u8>]) -> Rgb<u8> {
    let mut min_dist = f32::MAX;
    let mut closest_color = palette[0];

    for &color in palette {
        let dist_r = (r - color[0] as f32).powi(2);
        let dist_g = (g - color[1] as f32).powi(2);
        let dist_b = (b - color[2] as f32).powi(2);
        let dist = (dist_r + dist_g + dist_b).sqrt();

        if dist < min_dist {
            min_dist = dist;
            closest_color = color;
        }
    }
    closest_color
}

fn get_val_6color(data: &[u8], index: usize) -> u8 {
    if (data[index] == 0x00) && (data[index + 1] == 0x00) && (data[index + 2] == 0x00) {
        return 0;
    };
    if (data[index] == 0xFF) && (data[index + 1] == 0xFF) && (data[index + 2] == 0xFF) {
        return 1;
    };
    if (data[index] == 0xFF) && (data[index + 1] == 0xFF) && (data[index + 2] == 0x00) {
        return 2;
    };
    if (data[index] == 0xFF) && (data[index + 1] == 0x00) && (data[index + 2] == 0x00) {
        return 3;
    };
    // if (data[index] == 0xFF) && (data[index + 1] == 0x00) && (data[index + 2] == 0x00) {
    //     return 4;
    // };
    if (data[index] == 0x00) && (data[index + 1] == 0x00) && (data[index + 2] == 0xFF) {
        return 5;
    };
    if (data[index] == 0x00) && (data[index + 1] == 0xFF) && (data[index + 2] == 0x00) {
        return 6;
    };

    println!("ERROR: No matching color for pixel at index {}", index);
    return 7;
}

pub fn prepeare_image_for_sending(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    for i in 0..(width * height) {
        let val = get_val_6color(data, i * 3);
        output.push(val);
    }
    output
}

pub fn split_into_left_right(data: &[u8], width: usize, height: usize) -> (Vec<u8>, Vec<u8>) {
    let mut left: Vec<u8> = Vec::new();
    let mut right: Vec<u8> = Vec::new();

    for y in 0..height {
        let base = y * width;
        for x in 0..width / 2 {
            left.push(data[base + x]);
            right.push(data[base + x + (width / 2)]);
        }
    }

    (left, right)
}

#[allow(dead_code)]
pub fn add_white_padding_around_edges(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    let (width, height) = img.dimensions();
    const PADDING: u32 = 50;

    let color = Rgb([255, 255, 255]);

    for x in 0..width {
        for y in 0..PADDING {
            img.put_pixel(x, y, color);
            img.put_pixel(x, height - 1 - y, color);
        }
    }

    for y in 0..height {
        for x in 0..PADDING {
            img.put_pixel(x, y, color);
            img.put_pixel(width - 1 - x, y, color);
        }
    }
}
