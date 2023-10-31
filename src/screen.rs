use std::ffi::CStr;
use std::fmt::Display;
use std::fs;
use std::path::Path;

use fontdue::Font;
use hidapi::{HidApi, HidError};
use image::imageops::{dither, BiLevel, FilterType};
use image::DynamicImage;
use itertools::Itertools;

use crate::data::{DataPacket, HidAdapter, PAYLOAD_SIZE};
use crate::utils::{get_bit_at_index, set_bit_at_index};

pub enum ImageSizing {
    Contain,
    Cover,
    Original,
}

pub struct OledScreen {
    width: usize,
    height: usize,
    data: Vec<u8>,
    _prev_packets: Option<Vec<DataPacket>>,
    device: Box<dyn HidAdapter>,
}

impl Display for OledScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .data
            .iter()
            .chunks(self.width / 8)
            .into_iter()
            .map(|row| row.map(|byte| format!("{byte:08b}")).join(""))
            .join("\n")
            .replace('0', "░")
            .replace('1', "▓");
        f.write_str(&string)
    }
}

impl OledScreen {
    pub fn from_path(device_path: &CStr, width: usize, height: usize) -> Result<Self, HidError> {
        let api = HidApi::new()?;
        let device = api.open_path(device_path)?;
        Ok(Self {
            data: vec![0; (width * height) / 8],
            device: Box::new(device),
            width,
            height,
            _prev_packets: None,
        })
    }

    pub fn from_id(
        vid: u16,
        pid: u16,
        usage_page: u16,
        width: usize,
        height: usize,
    ) -> Result<Self, HidError> {
        let api = HidApi::new()?;

        let device_info = api.device_list().find(|dev| {
            dev.vendor_id() == vid && dev.product_id() == pid && dev.usage_page() == usage_page
        });
        if let Some(device_info) = device_info {
            let device = device_info.open_device(&api)?;
            Ok(Self {
                data: vec![0; (width * height) / 8],
                device: Box::new(device),
                width,
                height,
                _prev_packets: None,
            })
        } else {
            Err(HidError::HidApiError {
                message: "Could not find specified device".into(),
            })
        }
    }

    pub fn from_device(
        device: impl HidAdapter + 'static + Clone,
        width: usize,
        height: usize,
    ) -> Result<Self, HidError> {
        Ok(Self {
            data: vec![0; (width * height) / 8],
            device: Box::new(device),
            width,
            height,
            _prev_packets: None,
        })
    }

    pub(crate) fn to_packets(&self) -> Vec<DataPacket> {
        self.data
            .iter()
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

    pub fn draw_image_file<P: AsRef<Path>>(
        &mut self,
        image_path: P,
        x: usize,
        y: usize,
        sizing: &ImageSizing,
    ) {
        let image = image::open(image_path).unwrap();
        self.draw_image(image, x, y, sizing)
    }

    pub fn draw_image(
        &mut self,
        mut image: DynamicImage,
        x: usize,
        y: usize,
        sizing: &ImageSizing,
    ) {
        match sizing {
            ImageSizing::Contain => image = image.resize(32, 128, FilterType::Lanczos3),
            ImageSizing::Cover => {
                let scaling = f32::max(
                    32_f32 / image.width() as f32,
                    128_f32 / image.height() as f32,
                );

                image = image.resize(
                    (image.width() as f32 * scaling) as u32,
                    (image.height() as f32 * scaling) as u32,
                    FilterType::Lanczos3,
                );
            }
            ImageSizing::Original => (),
        };

        let mut image = image.grayscale().into_luma8();
        dither(&mut image, &BiLevel);

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
            let width = metrics.width;
            let height = metrics.height;

            let col = x + (index % width);
            let row = y + height - (index / width);
            let enabled = (byte as f32 / 255.0).round() as i32 == 1;
            self.set_pixel(col, row, enabled)
        }
    }

    pub fn send(&mut self) -> Result<(), HidError> {
        let mut packets = self.to_packets();

        // Filter out packets for regions of the screen which haven't changed since last time
        if let Some(prev_packets) = &self._prev_packets {
            packets.retain(|packet| !prev_packets.contains(packet))
        };

        self._prev_packets = Some(self.to_packets());

        for packet in packets {
            packet.send(self.device.as_ref())?;
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.data = vec![0; (self.width * self.height) / 8_usize];
    }

    pub fn fill_all(&mut self) {
        self.data = vec![1; (self.width * self.height) / 8_usize];
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
        let byte_index = (x + y * self.width) / 8;
        let bit_index: u8 = 7 - ((x % 8) as u8);

        let byte = self.data[byte_index];
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
        if x >= self.width || y >= self.height {
            // If a pixel is rendered outside of the canvas, fail silently
            return;
        }

        let x = x as usize;
        let y = y as usize;

        let target_byte = (x / 8) * self.height + y;
        let target_bit: u8 = 7 - ((x % 8) as u8);

        self.data[target_byte] = set_bit_at_index(self.data[target_byte], target_bit, enabled);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[derive(Clone)]
    struct MockHidDevice {
        pub write_log: RefCell<Vec<Vec<u8>>>,
    }

    impl MockHidDevice {
        pub const fn new() -> Self {
            MockHidDevice {
                write_log: RefCell::new(vec![]),
            }
        }
    }

    impl HidAdapter for MockHidDevice {
        fn write(&self, data: &[u8]) -> Result<usize, HidError> {
            self.write_log.borrow_mut().push(data.into());
            Ok(1)
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_display_oled_screen() {
        let mock_device = MockHidDevice::new();
        let mut screen = OledScreen::from_device(mock_device, 32, 128).unwrap();
        for i in 0..128 {
            screen.set_pixel(0, i, true);
            screen.set_pixel(31, i, true);
        }
        // FIXME: ASSERT
        println!("{screen}")
    }

    #[test]
    fn test_to_packets() {
        let mock_device = MockHidDevice::new();
        let screen = OledScreen::from_device(mock_device, 32, 128).unwrap();
        screen.to_packets();
        // FIXME: ASSERT
    }

    #[test]
    fn test_draw_image_file() {
        let mock_device = MockHidDevice::new();
        let mut screen = OledScreen::from_device(mock_device, 32, 128).unwrap();
        screen.draw_image_file(
            "assets/bitmaps/test_square.bmp",
            0,
            0,
            &ImageSizing::Contain,
        );
        println!("{screen}")
        // FIXME: ASSERT
    }

    #[test]
    fn test_draw_text() {
        let mock_device = MockHidDevice::new();
        let mut screen = OledScreen::from_device(mock_device, 32, 128).unwrap();
        screen.draw_text("Hey", 0, 0, 8.0, None);

        println!("{screen}");

        assert_eq!(
            screen.data,
            vec![
                0, 136, 8, 138, 138, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 128, 227, 129, 128,
                128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_packet_filtering() {
        let mock_device = MockHidDevice::new();
        let mut screen = OledScreen::from_device(mock_device, 32, 128).unwrap();
        screen.fill_all();
        screen.send().unwrap();
        screen.fill_all();
        screen.send().unwrap();

        let device: &MockHidDevice = screen
            .device
            .as_any()
            .downcast_ref::<MockHidDevice>()
            .unwrap();

        assert_eq!(18, device.write_log.borrow().len());
    }
}
