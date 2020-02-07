use super::*;

impl<N: Unsigned + Clone> SszEncode for Bitfield<length::Variable<N>> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl<N: Unsigned + Clone> SszDecode for Bitfield<length::Variable<N>> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        Self::from_bytes(bytes.to_vec()).map_err(|e| {
            SszDecodeError::BytesInvalid(format!("Failed while creating BitList: {:?}", e))
        })
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl<N: Unsigned + Clone> SszEncode for Bitfield<length::Fixed<N>> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }
}

impl<N: Unsigned + Clone> SszDecode for Bitfield<length::Fixed<N>> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        Self::from_bytes(bytes.to_vec()).map_err(|e| {
            SszDecodeError::BytesInvalid(format!("Failed while creating BitVector: {:?}", e))
        })
    }

    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_fixed_len() -> usize {
        bit_len_in_bytes_len(N::to_usize())
    }
}

fn bit_len_in_bytes_len(bit_len: usize) -> usize {
    std::cmp::max(1, (bit_len + 7) / 8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;

    #[test]
    fn len_conversions() {
        assert_eq!(bit_len_in_bytes_len(3), 1);
        assert_eq!(bit_len_in_bytes_len(8), 1);
        assert_eq!(bit_len_in_bytes_len(9), 2);
        assert_eq!(bit_len_in_bytes_len(15), 2);
        assert_eq!(bit_len_in_bytes_len(17), 3);
    }

    mod bitlist {
        use super::*;

        type BitList0 = Bitfield<length::Variable<U0>>;
        type BitList1 = Bitfield<length::Variable<U1>>;
        type BitList8 = Bitfield<length::Variable<U8>>;
        type BitList16 = Bitfield<length::Variable<U16>>;
        type BitList1024 = Bitfield<length::Variable<U1024>>;

        #[test]
        fn encode() {
            assert_eq!(
                BitList0::with_capacity(0).expect("Test").as_ssz_bytes(),
                vec![0b0_0000_0001],
            );

            assert_eq!(
                BitList1::with_capacity(0).expect("Test").as_ssz_bytes(),
                vec![0b0_0000_0001],
            );

            assert_eq!(
                BitList1::with_capacity(1).expect("Test").as_ssz_bytes(),
                vec![0b0_0000_0010],
            );

            assert_eq!(
                BitList8::with_capacity(8).expect("Test").as_ssz_bytes(),
                vec![0b0000_0000, 0b0000_0001],
            );

            assert_eq!(
                BitList8::with_capacity(7).expect("Test").as_ssz_bytes(),
                vec![0b1000_0000]
            );

            let mut b = BitList8::with_capacity(8).expect("Test");
            for i in 0..8 {
                b.set(i, true).expect("Test");
            }
            assert_eq!(b.as_ssz_bytes(), vec![255, 0b0000_0001]);

            let mut b = BitList8::with_capacity(8).expect("Test");
            for i in 0..4 {
                b.set(i, true).expect("Test");
            }
            assert_eq!(b.as_ssz_bytes(), vec![0b0000_1111, 0b0000_0001]);

            assert_eq!(
                BitList16::with_capacity(16).expect("Test").as_ssz_bytes(),
                vec![0b0000_0000, 0b0000_0000, 0b0000_0001]
            );
        }

        #[test]
        fn decode() {
            assert!(BitList0::from_ssz_bytes(&[]).is_err());
            assert!(BitList1::from_ssz_bytes(&[]).is_err());
            assert!(BitList8::from_ssz_bytes(&[]).is_err());
            assert!(BitList16::from_ssz_bytes(&[]).is_err());

            assert!(BitList0::from_ssz_bytes(&[0b0000_0000]).is_err());
            assert!(BitList1::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).is_err());
            assert!(BitList8::from_ssz_bytes(&[0b0000_0000]).is_err());
            assert!(BitList16::from_ssz_bytes(&[0b0000_0000]).is_err());

            assert!(BitList0::from_ssz_bytes(&[0b0000_0001]).is_ok());
            assert!(BitList0::from_ssz_bytes(&[0b0000_0010]).is_err());

            assert!(BitList1::from_ssz_bytes(&[0b0000_0001]).is_ok());
            assert!(BitList1::from_ssz_bytes(&[0b0000_0010]).is_ok());
            assert!(BitList1::from_ssz_bytes(&[0b0000_0100]).is_err());

            assert!(BitList8::from_ssz_bytes(&[0b0000_0001]).is_ok());
            assert!(BitList8::from_ssz_bytes(&[0b0000_0010]).is_ok());
            assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0001]).is_ok());
            assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0010]).is_err());
            assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0100]).is_err());
        }

        #[test]
        fn decode_extra_bytes() {
            assert!(BitList0::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
            assert!(BitList1::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
            assert!(BitList8::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
            assert!(BitList16::from_ssz_bytes(&[0b0000_0001, 0b0000_0000]).is_err());
            assert!(BitList1024::from_ssz_bytes(&[0b1000_0000, 0]).is_err());
            assert!(BitList1024::from_ssz_bytes(&[0b1000_0000, 0, 0]).is_err());
            assert!(BitList1024::from_ssz_bytes(&[0b1000_0000, 0, 0, 0, 0]).is_err());
        }

        #[test]
        fn ssz_round_trip() {
            assert_round_trip(&BitList0::with_capacity(0).expect("Test"));

            for i in 0..2 {
                assert_round_trip(&BitList1::with_capacity(i).expect("Test"));
            }
            for i in 0..9 {
                assert_round_trip(&BitList8::with_capacity(i).expect("Test"));
            }
            for i in 0..17 {
                assert_round_trip(&BitList16::with_capacity(i).expect("Test"));
            }

            let mut b = BitList1::with_capacity(1).expect("Test");
            b.set(0, true).expect("Test");
            assert_round_trip(&b);

            for i in 0..8 {
                let mut b = BitList8::with_capacity(i).expect("Test");
                for j in 0..i {
                    if j % 2 == 0 {
                        b.set(j, true).expect("Test");
                    }
                }
                assert_round_trip(&b);

                let mut b = BitList8::with_capacity(i).expect("Test");
                for j in 0..i {
                    b.set(j, true).expect("Test");
                }
                assert_round_trip(&b);
            }

            for i in 0..16 {
                let mut b = BitList16::with_capacity(i).expect("Test");
                for j in 0..i {
                    if j % 2 == 0 {
                        b.set(j, true).expect("Test");
                    }
                }
                assert_round_trip(&b);

                let mut b = BitList16::with_capacity(i).expect("Test");
                for j in 0..i {
                    b.set(j, true).expect("Test");
                }
                assert_round_trip(&b);
            }
        }
    }

    mod bitvector {
        use super::*;

        type BitVector0 = BitVector<U0>;
        type BitVector1 = BitVector<U1>;
        type BitVector4 = BitVector<U4>;
        type BitVector8 = BitVector<U8>;
        type BitVector16 = BitVector<U16>;

        #[test]
        fn encode() {
            assert_eq!(BitVector0::new().as_ssz_bytes(), vec![0b0000_0000]);
            assert_eq!(BitVector1::new().as_ssz_bytes(), vec![0b0000_0000]);
            assert_eq!(BitVector4::new().as_ssz_bytes(), vec![0b0000_0000]);
            assert_eq!(BitVector8::new().as_ssz_bytes(), vec![0b0000_0000]);
            assert_eq!(
                BitVector16::new().as_ssz_bytes(),
                vec![0b0000_0000, 0b0000_0000]
            );

            let mut b = BitVector8::new();
            for i in 0..8 {
                b.set(i, true).expect("Test");
            }
            assert_eq!(b.as_ssz_bytes(), vec![255]);

            let mut b = BitVector4::new();
            for i in 0..4 {
                b.set(i, true).expect("Test");
            }
            assert_eq!(b.as_ssz_bytes(), vec![0b0000_1111]);
        }

        #[test]
        fn decode() {
            assert!(BitVector0::from_ssz_bytes(&[0b0000_0000]).is_ok());
            assert!(BitVector0::from_ssz_bytes(&[0b0000_0001]).is_err());
            assert!(BitVector0::from_ssz_bytes(&[0b0000_0010]).is_err());

            assert!(BitVector1::from_ssz_bytes(&[0b0000_0001]).is_ok());
            assert!(BitVector1::from_ssz_bytes(&[0b0000_0010]).is_err());
            assert!(BitVector1::from_ssz_bytes(&[0b0000_0100]).is_err());
            assert!(BitVector1::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).is_err());

            assert!(BitVector8::from_ssz_bytes(&[0b0000_0000]).is_ok());
            assert!(BitVector8::from_ssz_bytes(&[1, 0b0000_0000]).is_err());
            assert!(BitVector8::from_ssz_bytes(&[0b0000_0000, 1]).is_err());
            assert!(BitVector8::from_ssz_bytes(&[0b0000_0001]).is_ok());
            assert!(BitVector8::from_ssz_bytes(&[0b0000_0010]).is_ok());
            assert!(BitVector8::from_ssz_bytes(&[0b0000_0100, 0b0000_0001]).is_err());
            assert!(BitVector8::from_ssz_bytes(&[0b0000_0100, 0b0000_0010]).is_err());
            assert!(BitVector8::from_ssz_bytes(&[0b0000_0100, 0b0000_0100]).is_err());

            assert!(BitVector16::from_ssz_bytes(&[0b0000_0000]).is_err());
            assert!(BitVector16::from_ssz_bytes(&[0b0000_0000, 0b0000_0000]).is_ok());
            assert!(BitVector16::from_ssz_bytes(&[1, 0b0000_0000, 0b0000_0000]).is_err());
        }

        #[test]
        fn ssz_round_trip() {
            assert_round_trip(&BitVector0::new());

            let mut b = BitVector1::new();
            b.set(0, true).expect("Test");
            assert_round_trip(&b);

            let mut b = BitVector8::new();
            for j in 0..8 {
                if j % 2 == 0 {
                    b.set(j, true).expect("Test");
                }
            }
            assert_round_trip(&b);

            let mut b = BitVector8::new();
            for j in 0..8 {
                b.set(j, true).expect("Test");
            }
            assert_round_trip(&b);

            let mut b = BitVector16::new();
            for j in 0..16 {
                if j % 2 == 0 {
                    b.set(j, true).expect("Test");
                }
            }
            assert_round_trip(&b);

            let mut b = BitVector16::new();
            for j in 0..16 {
                b.set(j, true).expect("Test");
            }
            assert_round_trip(&b);
        }
    }

    fn assert_round_trip<T: SszEncode + SszDecode + PartialEq + std::fmt::Debug>(t: &T) {
        assert_eq!(&T::from_ssz_bytes(&t.as_ssz_bytes()).expect("Test"), t);
    }
}
