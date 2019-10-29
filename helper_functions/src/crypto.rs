use ring::digest::{digest, SHA256};

pub fn hash(input: &[u8]) -> Vec<u8> {
    digest(&SHA256, input).as_ref().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing() {
        let input = b"lorem ipsum";
        let output = hash(input.as_ref());
        let output_bytes = output.as_ref();

        let expected_bytes = [
            0x5e, 0x2b, 0xf5, 0x7d, 0x3f, 0x40, 0xc4, 0xb6, 0xdf, 0x69, 0xda, 0xf1, 0x93, 0x6c,
            0xb7, 0x66, 0xf8, 0x32, 0x37, 0x4b, 0x4f, 0xc0, 0x25, 0x9a, 0x7c, 0xbf, 0xf0, 0x6e,
            0x2f, 0x70, 0xf2, 0x69,
        ];

        assert_eq!(expected_bytes, output_bytes);
    }
}
