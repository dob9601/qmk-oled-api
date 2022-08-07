use std::fmt::Display;

use hidapi::{HidDevice, HidError};
use itertools::Itertools;

fn set_bit_at_index(byte: u8, bit_index: u8, enabled: bool) -> u8 {
    let mask = 0b10000000 >> bit_index;

    if enabled {
        mask | byte
    } else {
        (mask ^ 0b11111111) & byte
    }
}

pub struct OledScreen32x128 {
    data: [[u8; 4]; 128],
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
            data: [[0; 4]; 128],
        }
    }

    pub fn to_packets(&self) -> Vec<DataPacket> {
        self.data
            .iter()
            .flatten()
            .chunks(3)
            .into_iter()
            .map(|chunk| {
                let bytes: Vec<&u8> = chunk.take(3).collect();
                (
                    **bytes.get(0).unwrap_or(&&0),
                    **bytes.get(1).unwrap_or(&&0),
                    **bytes.get(2).unwrap_or(&&0),
                )
            })
            .enumerate()
            .map(|(index, chunk)| {
                DataPacket::new(index.try_into().unwrap(), (chunk.0, chunk.1, chunk.2))
            })
            .collect()
    }

    pub fn send(&self, device: &HidDevice) -> Result<(), HidError> {
        let packets = self.to_packets();

        for packet in packets {
            packet.send(device)?;
        }

        Ok(())
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, enabled: bool) {
        let target_byte = x / 8;
        let target_bit = (x % 8).try_into().unwrap();

        self.data[y][target_byte] =
            set_bit_at_index(self.data[y][target_byte], target_bit, enabled);
    }
}

impl Default for OledScreen32x128 {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DataPacket {
    index: u8,
    payload: (u8, u8, u8),
}

impl DataPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.index, self.payload.0, self.payload.1, self.payload.2]
    }

    pub fn send(&self, device: &HidDevice) -> Result<(), HidError> {
        let bytes = self.to_bytes();

        device.write(&bytes)?;

        Ok(())
    }

    pub fn new(starting_index: u8, payload: (u8, u8, u8)) -> Self {
        Self {
            index: starting_index,
            payload: (payload.0, payload.1, payload.2),
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
            screen.set_pixel(0, i, true);
            screen.set_pixel(31, i, true);
        }
        println!("{screen}");
    }

    #[test]
    fn test_to_packets() {
        let screen = OledScreen32x128::new();
        screen.to_packets();
    }
}
