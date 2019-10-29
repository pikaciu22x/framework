// endianness is not configurable
pub fn int_to_bytes(int: u64, length: usize) -> Vec<u8> {
    let mut vec = int.to_le_bytes().to_vec();
    vec.resize(length, 0);
    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_to_bytes_value0_length_8() {
        let expected_bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let output = int_to_bytes(0, 8);
        let calculated_bytes = output.as_ref();

        assert_eq!(expected_bytes, calculated_bytes);
    }

    #[test]
    fn test_int_to_bytes_value2521273052_length_8() {
        let expected_bytes = [0xdc, 0x92, 0x47, 0x96, 0x00, 0x00, 0x00, 0x00];
        let output = int_to_bytes(2_521_273_052, 8);
        let calculated_bytes = output.as_ref();

        assert_eq!(expected_bytes, calculated_bytes);
    }

    #[test]
    fn test_int_to_bytes_value4294967295_length_8() {
        let expected_bytes = [0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00];
        let output = int_to_bytes(4_294_967_295, 8);
        let calculated_bytes = output.as_ref();

        assert_eq!(expected_bytes, calculated_bytes);
    }

    #[test]
    fn test_int_to_bytes_value88813769_length_32() {
        let expected_bytes = [
            0xc9, 0x30, 0x4b, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        let output = int_to_bytes(88_813_769, 32);
        let calculated_bytes = output.as_ref();

        assert_eq!(expected_bytes, calculated_bytes);
    }
}
