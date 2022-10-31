use std::any::Any;

use hidapi::{HidDevice, HidError};

pub trait HidAdapter {
    fn write(&self, data: &[u8]) -> Result<usize, HidError>;

    fn as_any(&self) -> &dyn Any;
}

impl HidAdapter for HidDevice {
    fn write(&self, data: &[u8]) -> Result<usize, HidError> {
        self.write(data)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The number of bytes in a payload. Typically this is 32.
pub const PAYLOAD_SIZE: usize = 32;

#[derive(PartialEq, Clone)]
pub(crate) struct DataPacket {
    index: u8,
    payload: [u8; PAYLOAD_SIZE - 2],
}

impl DataPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![1, self.index];
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    pub fn send(&self, device: &dyn HidAdapter) -> Result<(), HidError> {
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
