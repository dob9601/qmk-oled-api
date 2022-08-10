use std::cmp::min;
use std::fmt::Display;
use std::fs;
use std::path::Path;

use fontdue::Font;
use hidapi::{HidDevice, HidError};
use image::imageops::{dither, BiLevel};
use itertools::Itertools;

const PAYLOAD_SIZE: usize = 32;

fn set_bit_at_index(byte: u8, bit_index: u8, enabled: bool) -> u8 {
    let mask = 0b10000000 >> bit_index;

    if enabled {
        mask | byte
    } else {
        (mask ^ 0b11111111) & byte
    }
}

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

    pub fn draw_image<P: AsRef<Path>>(&mut self, bitmap_file: P, x: usize, y: usize) {
        let image = image::open(bitmap_file).unwrap();
        let mut image = image.grayscale();
        let image = image.as_mut_luma8().unwrap();
        dither(image, &BiLevel);

        let image_width = image.width();

        for (index, pixel) in image.pixels().enumerate() {
            let row = index / image_width as usize;
            let col = index % image_width as usize;

            let enabled = pixel.0[0] == 255;

            self.draw_pixel(x + row, y + col, enabled)
        }
    }

    pub fn draw_text<P: AsRef<Path>>(&mut self, font_path: P, text: &str, x: usize, y: usize, size: f32) {
        let font_bytes = fs::read(&font_path).unwrap();
        let font = Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap();

        let mut x_cursor = x;

        for letter in text.chars() {
            let width = font.metrics(letter, size).width;
            self.draw_letter(&font_path, letter, x_cursor, y, size);

            // FIXME: Use horizontal kerning as opposed to abstract value of "2"
            x_cursor += width + 2
        }
    }

    pub fn draw_letter<P: AsRef<Path>>(&mut self, font_path: P, letter: char, x: usize, y: usize, size: f32) {
        let font_bytes = fs::read(font_path).unwrap();
        let font = Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap();
        let (metrics, bitmap) = font.rasterize(letter, size);

        for (index, byte) in bitmap.into_iter().enumerate() {
            let col = x + (index % metrics.width);
            let row = y + metrics.height - (index / metrics.width);
            let enabled = (byte as f32 / 255.0).round() as i32 == 1;
            self.draw_pixel(col, row, enabled)
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

    pub fn paint_region(&mut self, min_x: usize, min_y: usize, max_x: usize, max_y: usize, enabled: bool) {
        for x in min_x..max_x {
            for y in min_y..max_y {
                self.draw_pixel(x, y, enabled)
            }
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, enabled: bool) {
        if x > 31 || y > 127 {
            // If a pixel is rendered outside of the canvas, fail silently
            return
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

pub struct DataPacket {
    index: u8,
    payload: [u8; PAYLOAD_SIZE - 2],
}

impl DataPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![1, self.index];
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn send(&self, device: &HidDevice) -> Result<(), HidError> {
        let bytes = self.to_bytes();

        device.write(&bytes)?;

        Ok(())
    }

    pub fn new(starting_index: u8, payload: [u8; PAYLOAD_SIZE - 2]) -> Self {
        Self {
            index: starting_index,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_bit_at_index() {
        let byte = 0b00000000;

        let output = set_bit_at_index(byte, 3, true);
        assert_eq!(output, 0b00010000)
    }

    #[test]
    fn test_disable_bit_at_index() {
        let byte = 0b11111111;

        let output = set_bit_at_index(byte, 3, false);
        assert_eq!(output, 0b11101111)
    }

    #[test]
    fn test_display_oled_screen() {
        let mut screen = OledScreen32x128::new();
        for i in 0..128 {
            screen.draw_pixel(0, i, true);
            screen.draw_pixel(31, i, true);
        }
        println!("{screen}");
    }

    #[test]
    fn test_to_packets() {
        let screen = OledScreen32x128::new();
        screen.to_packets();
    }

    #[test]
    fn test_draw_image() {
        let mut screen = OledScreen32x128::new();
        screen.draw_image("/home/dob9601/repos/qmk_nowplaying/w3c_home.bmp", 0, 0)
    }

    #[test]
    fn test_draw_text() {
        let mut screen = OledScreen32x128::new();
        screen.draw_letter("/home/dob9601/Downloads/Minecraft.ttf", 0, 0, 8.0)
    }
}
