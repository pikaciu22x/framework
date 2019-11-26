// endianness is not configurable
pub fn int_to_bytes(int: u64, length: usize) -> Vec<u8> {
    let mut vec = int.to_le_bytes().to_vec();
    vec.resize(length, 0);
    vec
}

pub fn int_to_bytes_32(int: u32, length: usize) -> Vec<u8> {
    let mut vec = int.to_le_bytes().to_vec();
    vec.resize(length, 0);
    vec
}

pub fn integer_squareroot(n: u64) -> u64 {
    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2
    }
    x
}

pub fn xor(bytes_1: &[u8; 32], bytes_2: &[u8; 32]) -> Vec<u8> {
    bytes_1
        .iter()
        .zip(bytes_2.iter())
        .map(|(a, b)| a ^ b)
        .collect()
}

#[allow(clippy::missing_const_for_fn)]
// todo: (bytes: &[u8]) -> Result<u64, Error>
pub fn bytes_to_int(bytes: [u8; 8]) -> u64 {
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::primitives::H256;

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
        let output = int_to_bytes(0xFFFF_FFFF, 8);
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

    #[test]
    fn test_int_to_bytes_32_value0_length_4() {
        let expected_bytes = [0x00, 0x00, 0x00, 0x00];
        let output = int_to_bytes_32(0, 4);
        let calculated_bytes = output.as_ref();

        assert_eq!(expected_bytes, calculated_bytes);
    }

    #[test]
    fn test_int_to_bytes_32_value4294967295_length_4() {
        let expected_bytes = [0xff, 0xff, 0xff, 0xff];
        let output = int_to_bytes_32(0xFFFF_FFFF, 4);
        let calculated_bytes = output.as_ref();

        assert_eq!(expected_bytes, calculated_bytes);
    }

    #[test]
    fn test_integer_squareroot() {
        assert_eq!(integer_squareroot(49), 7);
        assert_eq!(integer_squareroot(1), 1);
        assert_eq!(integer_squareroot(25), 5);
        assert_eq!(integer_squareroot(20), 4);
    }

    // todo:
    // #[test]
    // fn test_xor() {
    //     assert_eq!(
    //         xor(&H256::from([1; 32]), &H256::from([2; 32])[..]),
    //         &H256::from([3; 32])[..]
    //     );
    // }

    #[test]
    fn test_bytes_to_int() {
        assert_eq!(
            bytes_to_int([0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            2
        );
        assert_eq!(
            bytes_to_int([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            0
        );
        assert_eq!(
            bytes_to_int([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]),
            0x0100_0000_0000_0000
        );
    }
}
