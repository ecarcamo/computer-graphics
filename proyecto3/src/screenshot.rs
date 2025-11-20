use image::{Rgb, RgbImage};

pub fn save_screenshot(buffer: &Vec<u32>, width: usize, height: usize, filename: &str) {
    let mut img = RgbImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let pixel = buffer[y * width + x];

            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;

            img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }

    img.save(filename).unwrap();
}
