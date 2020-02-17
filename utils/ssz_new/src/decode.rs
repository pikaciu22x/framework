#![allow(clippy::use_self)]

use crate::*;
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

macro_rules! decode_for_uintn {
    ( $(($type_ident: ty, $size_in_bits: expr)),* ) => { $(
        impl SszDecode for $type_ident {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
                if bytes.len() == <Self as SszDecode>::ssz_fixed_len() {
                    let mut arr = [0; $size_in_bits / 8];
                    arr.clone_from_slice(bytes);
                    Ok(<$type_ident>::from_le_bytes(arr))
                } else {
                    Err(SszDecodeError::InvalidByteLength {
                        len: bytes.len(),
                        expected: <Self as SszDecode>::ssz_fixed_len(),
                    })
                }
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size_in_bits / 8
            }
        }
    )* };
}

decode_for_uintn!(
    (u8, 8),
    (u16, 16),
    (u32, 32),
    (u64, 64),
    (usize, std::mem::size_of::<usize>() * 8)
);

macro_rules! decode_for_u8_array {
    ($size: expr) => {
        impl SszDecode for [u8; $size] {
            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
                if bytes.len() == <Self as SszDecode>::ssz_fixed_len() {
                    let mut array: [u8; $size] = [0; $size];
                    array.copy_from_slice(&bytes[..]);

                    Ok(array)
                } else {
                    Err(SszDecodeError::InvalidByteLength {
                        len: bytes.len(),
                        expected: <Self as SszDecode>::ssz_fixed_len(),
                    })
                }
            }

            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                $size
            }
        }
    };
}

decode_for_u8_array!(4);
decode_for_u8_array!(32);

impl SszDecode for bool {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        if bytes.len() == <Self as SszDecode>::ssz_fixed_len() {
            match bytes[0] {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SszDecodeError::BytesInvalid(format!(
                    "Cannot deserialize bool from {}",
                    bytes[0]
                ))),
            }
        } else {
            Err(SszDecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: <Self as SszDecode>::ssz_fixed_len(),
            })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        1
    }
}

impl<T: SszDecode> SszDecode for Vec<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        let bytes_len = bytes.len();
        let fixed_len = <T as SszDecode>::ssz_fixed_len();

        if bytes.is_empty() {
            Ok(vec![])
        } else if !T::is_ssz_fixed_len() {
            decode_variable_sized_items(bytes)
        } else if bytes_len % fixed_len == 0 {
            let mut result = Vec::with_capacity(bytes.len() / fixed_len);
            for chunk in bytes.chunks(fixed_len) {
                result.push(T::from_ssz_bytes(chunk)?);
            }

            Ok(result)
        } else {
            Err(SszDecodeError::InvalidByteLength {
                len: bytes_len,
                expected: bytes.len() / <T as SszDecode>::ssz_fixed_len() + 1,
            })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl SszDecode for NonZeroUsize {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        let val = usize::from_ssz_bytes(bytes)?;

        if val == 0 {
            Err(SszDecodeError::BytesInvalid(
                "NonZeroUsize cannot be zero.".to_string(),
            ))
        } else {
            Ok(NonZeroUsize::new(val).expect("0 check is done above"))
        }
    }

    fn is_ssz_fixed_len() -> bool {
        <usize as SszDecode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <usize as SszDecode>::ssz_fixed_len()
    }
}

impl<T: SszDecode> SszDecode for Option<T> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        if bytes.len() < BYTES_PER_LENGTH_OFFSET {
            return Err(SszDecodeError::InvalidByteLength {
                len: bytes.len(),
                expected: BYTES_PER_LENGTH_OFFSET,
            });
        }

        let (index_bytes, value_bytes) = bytes.split_at(BYTES_PER_LENGTH_OFFSET);

        let index = decode_offset(index_bytes)?;
        if index == 0 {
            Ok(None)
        } else if index == 1 {
            Ok(Some(T::from_ssz_bytes(value_bytes)?))
        } else {
            Err(SszDecodeError::BytesInvalid(format!(
                "{} is not a valid union index for Option<T>",
                index
            )))
        }
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl SszDecode for H256 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        let len = bytes.len();
        let expected = <Self as SszDecode>::ssz_fixed_len();

        if len == expected {
            Ok(H256::from_slice(bytes))
        } else {
            Err(SszDecodeError::InvalidByteLength { len, expected })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }
}

impl SszDecode for U256 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        let len = bytes.len();
        let expected = <Self as SszDecode>::ssz_fixed_len();

        if len == expected {
            Ok(U256::from_little_endian(bytes))
        } else {
            Err(SszDecodeError::InvalidByteLength { len, expected })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        32
    }
}

impl SszDecode for U128 {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        let len = bytes.len();
        let expected = <Self as SszDecode>::ssz_fixed_len();

        if len == expected {
            Ok(U128::from_little_endian(bytes))
        } else {
            Err(SszDecodeError::InvalidByteLength { len, expected })
        }
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8() {
        assert_eq!(u8::from_ssz_bytes(&[0b0000_0000]).expect("Test"), 0);
        assert_eq!(
            u8::from_ssz_bytes(&[0b1111_1111]).expect("Test"),
            u8::max_value()
        );
        assert_eq!(u8::from_ssz_bytes(&[0b0000_0001]).expect("Test"), 1);
        assert_eq!(u8::from_ssz_bytes(&[0b1000_0000]).expect("Test"), 128);

        assert!(u8::from_ssz_bytes(&[]).is_err());
        assert!(u8::from_ssz_bytes(&[0; 2]).is_err());

        assert_eq!(<u8 as SszDecode>::ssz_fixed_len(), 1);
    }

    #[test]
    fn u16() {
        assert_eq!(
            u16::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).expect("Test"),
            0
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).expect("Test"),
            1
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b1000_0000, 0b0000_0000]).expect("Test"),
            128
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b1111_1111, 0b1111_1111]).expect("Test"),
            u16::max_value()
        );
        assert_eq!(
            u16::from_ssz_bytes(&[0b0000_0000, 0b1000_0000]).expect("Test"),
            0x8000
        );

        assert!(u16::from_ssz_bytes(&[]).is_err());
        assert!(u16::from_ssz_bytes(&[0; 1]).is_err());
        assert!(u16::from_ssz_bytes(&[0; 3]).is_err());

        assert_eq!(<u16 as SszDecode>::ssz_fixed_len(), 2);
    }

    #[test]
    fn u32() {
        assert_eq!(u32::from_ssz_bytes(&[0b0000_0000; 4]).expect("Test"), 0);
        assert_eq!(
            u32::from_ssz_bytes(&[0b1111_1111; 4]).expect("Test"),
            u32::max_value()
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000])
                .expect("Test"),
            1
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000])
                .expect("Test"),
            128
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000])
                .expect("Test"),
            0x8000
        );
        assert_eq!(
            u32::from_ssz_bytes(&[0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000])
                .expect("Test"),
            0x8000_0000
        );

        assert!(u32::from_ssz_bytes(&[]).is_err());
        assert!(u32::from_ssz_bytes(&[0; 2]).is_err());
        assert!(u32::from_ssz_bytes(&[0; 5]).is_err());

        assert_eq!(<u32 as SszDecode>::ssz_fixed_len(), 4);
    }

    #[test]
    fn u64() {
        assert_eq!(u64::from_ssz_bytes(&[0b0000_0000; 8]).expect("Test"), 0);
        assert_eq!(
            u64::from_ssz_bytes(&[0b1111_1111; 8]).expect("Test"),
            u64::max_value()
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0001,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            1
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            128
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            0x8000
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ])
            .expect("Test"),
            0x8000_0000
        );
        assert_eq!(
            u64::from_ssz_bytes(&[
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1000_0000
            ])
            .expect("Test"),
            0x8000_0000_0000_0000
        );

        assert!(u64::from_ssz_bytes(&[]).is_err());
        assert!(u64::from_ssz_bytes(&[0; 2]).is_err());
        assert!(u64::from_ssz_bytes(&[0; 9]).is_err());

        assert_eq!(<u64 as SszDecode>::ssz_fixed_len(), 8);
    }

    #[test]
    fn usize() {
        let usize_size = std::mem::size_of::<usize>();

        assert_eq!(
            usize::from_ssz_bytes(&vec![0b0000_0000; usize_size]).expect("Test"),
            0
        );
        assert_eq!(
            usize::from_ssz_bytes(&vec![0b1111_1111; usize_size]).expect("Test"),
            usize::max_value()
        );

        assert!(usize::from_ssz_bytes(&[]).is_err());
        assert!(usize::from_ssz_bytes(&[0; 2]).is_err());
        assert!(usize::from_ssz_bytes(&[0; 9]).is_err());

        assert_eq!(<usize as SszDecode>::ssz_fixed_len(), usize_size)
    }

    #[test]
    fn non_zero_usize() {
        let usize_size = std::mem::size_of::<usize>();

        assert_eq!(
            NonZeroUsize::from_ssz_bytes(&vec![0b1111_1111; usize_size])
                .expect("Test")
                .get(),
            usize::max_value()
        );

        assert!(NonZeroUsize::from_ssz_bytes(&[0; 8]).is_err());

        assert!(NonZeroUsize::from_ssz_bytes(&[]).is_err());
        assert!(NonZeroUsize::from_ssz_bytes(&[0; 2]).is_err());
        assert!(NonZeroUsize::from_ssz_bytes(&[0; 9]).is_err());

        assert_eq!(<NonZeroUsize as SszDecode>::ssz_fixed_len(), usize_size);
        assert!(<NonZeroUsize as SszDecode>::is_ssz_fixed_len());
    }

    #[test]
    fn u8_array() {
        assert_eq!(<[u8; 4]>::from_ssz_bytes(&[0; 4]).expect("Test"), [0; 4]);
        assert_eq!(<[u8; 32]>::from_ssz_bytes(&[0; 32]).expect("Test"), [0; 32]);
        assert_eq!(
            <[u8; 4]>::from_ssz_bytes(&[u8::max_value(); 4]).expect("Test"),
            [u8::max_value(); 4]
        );
        assert_eq!(
            <[u8; 32]>::from_ssz_bytes(&[u8::max_value(); 32]).expect("Test"),
            [u8::max_value(); 32]
        );

        assert!(<[u8; 4]>::from_ssz_bytes(&[0; 5]).is_err());
        assert!(<[u8; 32]>::from_ssz_bytes(&[0; 34]).is_err());

        assert_eq!(<[u8; 4] as SszDecode>::ssz_fixed_len(), 4);
        assert_eq!(<[u8; 32] as SszDecode>::ssz_fixed_len(), 32);

        assert!(<[u8; 4] as SszDecode>::is_ssz_fixed_len());
        assert!(<[u8; 32] as SszDecode>::is_ssz_fixed_len());
    }

    #[test]
    fn bool() {
        assert_eq!(bool::from_ssz_bytes(&[0_u8]).expect("Test"), false);
        assert_eq!(bool::from_ssz_bytes(&[1_u8]).expect("Test"), true);

        assert!(bool::from_ssz_bytes(&[2_u8]).is_err());
        assert!(bool::from_ssz_bytes(&[0_u8, 0_u8]).is_err());

        assert!(<bool as SszDecode>::is_ssz_fixed_len());
        assert_eq!(<bool as SszDecode>::ssz_fixed_len(), 1);
    }

    #[test]
    fn option() {
        let none: Option<u16> = None;

        assert_eq!(
            <Option<u16>>::from_ssz_bytes(&[1, 0, 0, 0, 42, 0]).expect("Test"),
            Some(42)
        );
        assert_eq!(<Option<u16>>::from_ssz_bytes(&[0; 4]).expect("Test"), none);

        assert!(<Option<u16>>::from_ssz_bytes(&[1, 0, 0]).is_err());
        assert!(<Option<u16>>::from_ssz_bytes(&[2, 0, 0, 0]).is_err());
        assert!(<Option<u16>>::from_ssz_bytes(&[1, 0, 0, 0]).is_err());

        assert!(!<Option<u16> as SszDecode>::is_ssz_fixed_len());
    }

    #[test]
    fn h256() {
        assert_eq!(H256::from_ssz_bytes(&[0; 32]).expect("Test"), H256::zero());

        assert!(H256::from_ssz_bytes(&[0; 31]).is_err());
        assert!(H256::from_ssz_bytes(&[0; 33]).is_err());

        assert!(<H256 as SszDecode>::is_ssz_fixed_len());
        assert_eq!(<H256 as SszDecode>::ssz_fixed_len(), 32)
    }

    #[test]
    fn u256() {
        assert_eq!(
            U256::from_ssz_bytes(&[0; 32]).expect("Test"),
            U256::from_dec_str("0").expect("Test")
        );

        assert!(U256::from_ssz_bytes(&[0; 31]).is_err());
        assert!(U256::from_ssz_bytes(&[0; 33]).is_err());

        assert!(<U256 as SszDecode>::is_ssz_fixed_len());
        assert_eq!(<U256 as SszDecode>::ssz_fixed_len(), 32)
    }

    #[test]
    fn u128() {
        assert_eq!(
            U128::from_ssz_bytes(&[0; 16]).expect("Test"),
            U128::from_dec_str("0").expect("Test")
        );

        assert!(U128::from_ssz_bytes(&[0; 15]).is_err());
        assert!(U128::from_ssz_bytes(&[0; 17]).is_err());

        assert!(<U128 as SszDecode>::is_ssz_fixed_len());
        assert_eq!(<U128 as SszDecode>::ssz_fixed_len(), 16)
    }

    #[test]
    fn vector() {
        assert!(<Vec<bool>>::from_ssz_bytes(&[0, 1, 2]).is_err());
        assert!(<Vec<u32>>::from_ssz_bytes(&[0, 1, 2, 4, 5]).is_err());

        assert!(!<Vec<u32> as SszDecode>::is_ssz_fixed_len());
    }

    #[test]
    fn vector_fixed() {
        assert_eq!(<Vec<u8>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u8>>::from_ssz_bytes(&[0, 1, 2, 3]).expect("Test"),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            <Vec<u8>>::from_ssz_bytes(&[u8::max_value(); 100]).expect("Test"),
            vec![u8::max_value(); 100]
        );

        assert_eq!(<Vec<u16>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u16>>::from_ssz_bytes(&[1, 0, 2, 0, 3, 0, 4, 0]).expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u16>>::from_ssz_bytes(&[u8::max_value(); 200]).expect("Test"),
            vec![u16::max_value(); 100]
        );

        assert_eq!(<Vec<u32>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u32>>::from_ssz_bytes(&[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0])
                .expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u32>>::from_ssz_bytes(&[u8::max_value(); 400]).expect("Test"),
            vec![u32::max_value(); 100]
        );

        assert_eq!(<Vec<u64>>::from_ssz_bytes(&[]).expect("Test"), vec![]);
        assert_eq!(
            <Vec<u64>>::from_ssz_bytes(&[
                1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
                0, 0, 0, 0
            ])
            .expect("Test"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            <Vec<u64>>::from_ssz_bytes(&[u8::max_value(); 800]).expect("Test"),
            vec![u64::max_value(); 100]
        );
    }

    #[test]
    fn vector_variable() {
        let vec: Vec<Vec<u8>> = vec![];
        assert_eq!(<Vec<Vec<u8>>>::from_ssz_bytes(&[]).expect("Test"), vec);

        let vec: Vec<Vec<u8>> = vec![vec![], vec![]];
        assert_eq!(
            <Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0]).expect("Test"),
            vec
        );

        let vec: Vec<Vec<u8>> = vec![vec![1, 2, 3], vec![4, 5, 6]];
        assert_eq!(
            <Vec<Vec<u8>>>::from_ssz_bytes(&[8, 0, 0, 0, 11, 0, 0, 0, 1, 2, 3, 4, 5, 6])
                .expect("Test"),
            vec
        );
    }

    #[test]
    fn vector_variable_error() {
        // incorrect bytes length for offset
        assert!(<Vec<Vec<u8>>>::from_ssz_bytes(&[0, 1, 2]).is_err());

        // offset is too large
        assert!(<Vec<Vec<u8>>>::from_ssz_bytes(&[10, 0, 0, 0, 2]).is_err());

        // too short value part
        assert!(<Vec<Vec<u64>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0, 1]).is_err());

        // wrong bytes to deserialize value
        assert!(<Vec<Vec<bool>>>::from_ssz_bytes(&[8, 0, 0, 0, 8, 0, 0, 0, 2]).is_err());
    }
}
