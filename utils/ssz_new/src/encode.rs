#![allow(clippy::use_self)]

use crate::utils::*;
use crate::*;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! encode_for_uintn {
    ( $(($type_ident: ty, $size_in_bits: expr)),* ) => { $(
        impl SszEncode for $type_ident {
            fn as_ssz_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }
        }
    )* };
}

encode_for_uintn!(
    (u8, 8),
    (u16, 16),
    (u32, 32),
    (u64, 64),
    (usize, std::mem::size_of::<usize>() * 8)
);

macro_rules! encode_for_u8_array {
    ($size: expr) => {
        impl SszEncode for [u8; $size] {
            fn as_ssz_bytes(&self) -> Vec<u8> {
                self.to_vec()
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }
        }
    };
}

encode_for_u8_array!(4);
encode_for_u8_array!(32);

impl SszEncode for bool {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let byte = if *self { 0b0000_0001 } else { 0b0000_0000 };
        vec![byte]
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }
}

impl<T: SszEncode> SszEncode for Vec<T> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut fixed_parts = Vec::with_capacity(self.len());
        for element in self {
            fixed_parts.push(if T::is_ssz_fixed_len() {
                Some(element.as_ssz_bytes())
            } else {
                None
            });
        }

        let mut variable_parts = Vec::with_capacity(self.len());
        for element in self {
            variable_parts.push(if T::is_ssz_fixed_len() {
                vec![]
            } else {
                element.as_ssz_bytes()
            });
        }

        encode_items_from_parts(&fixed_parts, &variable_parts)
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl<T: SszEncode> SszEncode for Option<T> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        match self {
            None => encode_offset(0),
            Some(t) => {
                let mut result = encode_offset(1);
                result.append(&mut t.as_ssz_bytes());

                result
            }
        }
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl SszEncode for NonZeroUsize {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.get().as_ssz_bytes()
    }

    fn is_ssz_fixed_len() -> bool {
        <usize as SszEncode>::is_ssz_fixed_len()
    }
}

impl SszEncode for H256 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }
}

impl SszEncode for U256 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = vec![0; 32];
        self.to_little_endian(&mut result);
        result
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }
}

impl SszEncode for U128 {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = vec![0; 16];
        self.to_little_endian(&mut result);
        result
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn u8() {
        assert_eq!(0_u8.as_ssz_bytes(), vec![0b0000_0000]);
        assert_eq!(u8::max_value().as_ssz_bytes(), vec![0b1111_1111]);
        assert_eq!(1_u8.as_ssz_bytes(), vec![0b0000_0001]);
        assert_eq!(128_u8.as_ssz_bytes(), vec![0b1000_0000]);

        assert!(<u8 as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn u16() {
        assert_eq!(0_u16.as_ssz_bytes(), vec![0b0000_0000, 0b0000_0000]);
        assert_eq!(1_u16.as_ssz_bytes(), vec![0b0000_0001, 0b0000_0000]);
        assert_eq!(128_u16.as_ssz_bytes(), vec![0b1000_0000, 0b0000_0000]);
        assert_eq!(
            u16::max_value().as_ssz_bytes(),
            vec![0b1111_1111, 0b1111_1111]
        );
        assert_eq!(0x8000_u16.as_ssz_bytes(), vec![0b0000_0000, 0b1000_0000]);

        assert!(<u16 as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn u32() {
        assert_eq!(0_u32.as_ssz_bytes(), vec![0b0000_0000; 4]);
        assert_eq!(u32::max_value().as_ssz_bytes(), vec![0b1111_1111; 4]);
        assert_eq!(
            1_u32.as_ssz_bytes(),
            vec![0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            128_u32.as_ssz_bytes(),
            vec![0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            0x8000_u32.as_ssz_bytes(),
            vec![0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000]
        );
        assert_eq!(
            0x8000_0000_u32.as_ssz_bytes(),
            vec![0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000]
        );

        assert!(<u32 as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn u64() {
        assert_eq!(0_u64.as_ssz_bytes(), vec![0b0000_0000; 8]);
        assert_eq!(u64::max_value().as_ssz_bytes(), vec![0b1111_1111; 8]);
        assert_eq!(
            1_u64.as_ssz_bytes(),
            vec![
                0b0000_0001,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            128_u64.as_ssz_bytes(),
            vec![
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            0x8000_u64.as_ssz_bytes(),
            vec![
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            0x8000_0000_u64.as_ssz_bytes(),
            vec![
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
        assert_eq!(
            0x8000_0000_0000_0000_u64.as_ssz_bytes(),
            vec![
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000
            ]
        );

        assert!(<u64 as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn usize() {
        let usize_size = std::mem::size_of::<usize>();

        let encoded = 1_usize.as_ssz_bytes();
        assert_eq!(encoded.len(), usize_size);
        for (i, byte) in encoded.iter().enumerate() {
            if i == 0 {
                assert_eq!(*byte, 1)
            } else {
                assert_eq!(*byte, 0)
            }
        }

        assert_eq!(usize::max_value().as_ssz_bytes(), vec![255; usize_size]);

        assert!(<usize as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn non_zero_usize() {
        let usize_size = std::mem::size_of::<usize>();

        let nzusize = NonZeroUsize::new(usize::max_value()).expect("Test");
        assert_eq!(nzusize.as_ssz_bytes(), vec![255; usize_size]);

        assert!(<NonZeroUsize as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn bool() {
        assert_eq!(true.as_ssz_bytes(), vec![0b0000_0001]);
        assert_eq!(false.as_ssz_bytes(), vec![0b0000_0000]);

        assert!(<bool as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn vector_fixed() {
        let vec: Vec<u8> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u8> = vec![0, 1, 2, 3];
        assert_eq!(vec.as_ssz_bytes(), vec![0, 1, 2, 3]);

        let vec: Vec<u8> = vec![u8::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 100]);

        let vec: Vec<u16> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u16> = vec![1, 2, 3, 4];
        assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let vec: Vec<u16> = vec![u16::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 200]);

        let vec: Vec<u32> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u32> = vec![1, 2, 3, 4];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );

        let vec: Vec<u32> = vec![u32::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 400]);

        let vec: Vec<u64> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<u64> = vec![1, 2, 3, 4];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![
                1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                0, 0, 0, 0
            ]
        );

        let vec: Vec<u64> = vec![u64::max_value(); 100];
        assert_eq!(vec.as_ssz_bytes(), vec![u8::max_value(); 800]);
        assert!(!<Vec<u64> as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(vec.as_ssz_bytes(), vec![]);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(vec.as_ssz_bytes(), vec![8, 0, 0, 0, 8, 0, 0, 0]);

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(
            vec.as_ssz_bytes(),
            vec![8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn option() {
        let some = Some(u16::max_value());
        assert_eq!(some.as_ssz_bytes(), vec![1, 0, 0, 0, 255, 255]);

        let none: Option<u16> = None;
        assert_eq!(none.as_ssz_bytes(), vec![0, 0, 0, 0]);
        assert!(!<Option<u16> as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn u8_array() {
        assert_eq!([1; 4].as_ssz_bytes(), vec![1; 4]);
        assert_eq!([1; 32].as_ssz_bytes(), vec![1; 32]);

        assert!(<[u8; 4] as SszEncode>::is_ssz_fixed_len());
        assert!(<[u8; 32] as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn h256() {
        assert_eq!(H256::zero().as_ssz_bytes(), vec![0; 32]);

        assert!(<H256 as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn u256() {
        let u = U256::from_dec_str("0").expect("Test");
        assert_eq!(u.as_ssz_bytes(), vec![0; 32]);

        assert!(<U256 as SszEncode>::is_ssz_fixed_len());
    }

    #[test]
    fn u128() {
        let u = U128::from_dec_str("0").expect("Test");
        assert_eq!(u.as_ssz_bytes(), vec![0; 16]);

        assert!(<U128 as SszEncode>::is_ssz_fixed_len());
    }
}
