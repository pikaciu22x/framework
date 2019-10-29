pub fn integer_squareroot(n: u64) -> u64 {
    let mut x = n;
    let mut y = (x + 1) / 2;

    while y < x {
        x = y;
        y = (x + n / x) / 2
    }
    x
}

pub fn bytes_to_int(bytes: [u8; 8]) -> u64 {
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_squareroot() {
        assert_eq!(integer_squareroot(49), 7);
        assert_eq!(integer_squareroot(1), 1);
        assert_eq!(integer_squareroot(25), 5);
        assert_eq!(integer_squareroot(20), 4);
    }

    #[test]
    fn test_bytes_to_int() {
        assert_eq!(bytes_to_int([0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]), 2);
        assert_eq!(bytes_to_int([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]), 0);
        assert_eq!(bytes_to_int([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]), 72057594037927936_u64);
    }
}
