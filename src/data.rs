use std::fmt::Display;

use hidapi::{HidDevice, HidError};
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

    pub fn send(&self, device: &HidDevice) -> Result<(), HidError> {
        let packets = self.to_packets();

        for packet in packets {
            packet.send(device)?;
        }

        Ok(())
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, enabled: bool) {
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
