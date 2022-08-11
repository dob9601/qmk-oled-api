use hidapi::{HidDevice, HidError};

/// The number of bytes in a payload. Typically this is 32.
pub const PAYLOAD_SIZE: usize = 32;

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
