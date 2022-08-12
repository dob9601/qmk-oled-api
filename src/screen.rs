use std::fmt::Display;
use std::fs;
use std::path::Path;

use fontdue::Font;
use hidapi::{HidDevice, HidError};
use image::imageops::{dither, BiLevel, resize, FilterType};
use itertools::Itertools;

use crate::data::{DataPacket, PAYLOAD_SIZE};
use crate::utils::{set_bit_at_index, get_bit_at_index};

pub struct OledScreen32x128 {
    data: [[u8; 128]; 4],
}

impl Display for OledScreen32x128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .data
            .iter()
            .map(|row| row.map(|byte| format!("{byte:08b}")).join(""))
            .join("\n")
            .replace('0', "░")
            .replace('1', "▓");
        f.write_str(&string)
    }
}

impl OledScreen32x128 {
    pub fn new() -> Self {
        Self {
            data: [[0; 128]; 4],
        }
    }

    pub fn to_packets(&self) -> Vec<DataPacket> {
        self.data
            .iter()
            .flatten()
            .chunks(PAYLOAD_SIZE - 2)
            .into_iter()
            .map(|chunk| {
                let mut output_array: [u8; PAYLOAD_SIZE - 2] = [0; PAYLOAD_SIZE - 2];
                chunk
                    .take(PAYLOAD_SIZE - 2)
                    .enumerate()
                    .for_each(|(index, byte)| output_array[index] = *byte);
                output_array
            })
            .enumerate()
            .map(|(index, chunk)| DataPacket::new(index.try_into().unwrap(), chunk))
            .collect()
    }

    pub fn draw_image<P: AsRef<Path>>(&mut self, bitmap_file: P, x: usize, y: usize, scale: bool) {
        let mut image = image::open(bitmap_file).unwrap();
        if scale {
            // TODO: Find a better way of specifying canvas size
            image = image.resize(32, 128, FilterType::Lanczos3);
        }

        let mut image = image.grayscale();
        let image = image.as_mut_luma8().unwrap();
        dither(image, &BiLevel);


        let image_width = image.width();
        let image_height = image.height();

        for (index, pixel) in image.pixels().enumerate() {
            let row = index / image_width as usize;
            let col = index % image_width as usize;

            let enabled = pixel.0[0] == 255;

            self.set_pixel(x + col, y + image_height as usize - row, enabled)
        }
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        x: usize,
        y: usize,
        size: f32,
        font_path: Option<&str>,
    ) {
        let font = if let Some(font_path) = font_path {
            let font_bytes = fs::read(&font_path).unwrap();
            Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap()
        } else {
            Font::from_bytes(
                include_bytes!("../assets/cozette.ttf") as &[u8],
                fontdue::FontSettings::default(),
            )
            .unwrap()
        };

        let mut x_cursor = x;

        for letter in text.chars() {
            let width = font.metrics(letter, size).width;
            self.draw_letter(letter, x_cursor, y, size, &font);

            // FIXME: Use horizontal kerning as opposed to abstract value of "2"
            x_cursor += width + 2
        }
    }

    fn draw_letter(&mut self, letter: char, x: usize, y: usize, size: f32, font: &Font) {
        let (metrics, bitmap) = font.rasterize(letter, size);

        for (index, byte) in bitmap.into_iter().enumerate() {
            let col = x + (index % metrics.width);
            let row = y + metrics.height - (index / metrics.width);
            let enabled = (byte as f32 / 255.0).round() as i32 == 1;
            self.set_pixel(col, row, enabled)
        }
    }

    pub fn send(&self, device: &HidDevice) -> Result<(), HidError> {
        let packets = self.to_packets();

        for packet in packets {
            packet.send(device)?;
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.data = [[0; 128]; 4];
    }

    pub fn fill_all(&mut self) {
        self.data = [[1; 128]; 4];
    }

    pub fn paint_region(
        &mut self,
        min_x: usize,
        min_y: usize,
        max_x: usize,
        max_y: usize,
        enabled: bool,
    ) {
        for x in min_x..max_x {
            for y in min_y..max_y {
                self.set_pixel(x, y, enabled)
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        let byte_index = x / 8;
        let bit_index: u8 = 7 - ((x % 8) as u8);

        let byte = self.data[byte_index][y];
        get_bit_at_index(byte, bit_index)
    }

    /// Underlying function for drawing to the canvas, if provided coordinates are out of range,
    /// this function will fail silently
    ///
    /// # Arguments
    /// * `x` - The x coordinate of the pixel to set
    /// * `y` - The y coordinate of the pixel to set
    /// * `enabled` - Whether to set the pixel to an enabled or disabled state (on/off)
    pub fn set_pixel(&mut self, x: usize, y: usize, enabled: bool) {
        if x > 31 || y > 127 {
            // If a pixel is rendered outside of the canvas, fail silently
            return;
        }

        let target_byte = x / 8;
        let target_bit: u8 = 7 - ((x % 8) as u8);

        self.data[target_byte][y] =
            set_bit_at_index(self.data[target_byte][y], target_bit, enabled);
    }
}

impl Default for OledScreen32x128 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_oled_screen() {
        let mut screen = OledScreen32x128::new();
        for i in 0..128 {
            screen.set_pixel(0, i, true);
            screen.set_pixel(31, i, true);
        }
        // FIXME: ASSERT
    }

    #[test]
    fn test_to_packets() {
        let screen = OledScreen32x128::new();
        screen.to_packets();
        // FIXME: ASSERT
    }

    #[test]
    fn test_draw_image() {
        let mut screen = OledScreen32x128::new();
        screen.draw_image("assets/bitmaps/test_square.bmp", 0, 0, false);
        // FIXME: ASSERT
    }

    #[test]
    fn test_draw_text() {
        let mut screen = OledScreen32x128::new();
        screen.draw_text("Hey", 0, 0, 8.0, None);

        assert_eq!(
            screen.data,
            [
                [
                    0, 136, 8, 138, 138, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 65, 128, 227, 129, 128, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ]
            ]
        );
    }
}
