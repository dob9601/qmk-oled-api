fn get_bit_at_index(byte: u8, bit_index: u8) -> bool {
    let mask = 0b10000000 >> bit_index;

    mask & byte != 0
}

pub fn set_bit_at_index(byte: u8, bit_index: u8, enabled: bool) -> u8 {
    let mask = 0b10000000 >> bit_index;

    if enabled {
        mask | byte
    } else {
        (mask ^ 0b11111111) & byte
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_bit_at_index() {
        let input = 0b00000000;
        let actual = set_bit_at_index(input, 3, true);

        assert_eq!(actual, 0b00010000)
    }

    #[test]
    fn test_get_bit_at_index() {
        let input = 0b00010000;
        let bit_state = get_bit_at_index(input, 3);

        assert!(bit_state)
    }
}
