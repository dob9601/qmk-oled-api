use itertools::Itertools;

struct OledScreen32x128 {
    data: [[bool; 32]; 128]
}

impl OledScreen32x128 {
    pub fn new() -> Self {
        Self {
            data: [[false; 32]; 128]
        }
    }

    pub fn to_packets(&self) -> Vec<DataPacket> {
        for chunk in &self.data.iter().flatten().chunks(24) {
            println!("{:?}", chunk.collect::<Vec<&bool>>());
        }

        todo!()
    }
}

pub struct DataPacket {
    index: u8,
    payload: (u8, u8, u8)
}

impl DataPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.index, self.payload.0, self.payload.1, self.payload.2]
    }

    pub fn new(starting_index: u8, payload: (u8, u8, u8)) -> Self {
        Self {
            index: starting_index,
            payload
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_packets() {
        let screen = OledScreen32x128::new();
        screen.to_packets();
    }
}
