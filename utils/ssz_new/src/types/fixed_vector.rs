use super::*;

impl<T: SszEncode, N: Unsigned> SszEncode for FixedVector<T, N> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        let mut result = vec![];

        if T::is_ssz_fixed_len() {
            for element in self.iter() {
                result.append(&mut element.as_ssz_bytes());
            }
        } else {
            let mut variable_parts = Vec::with_capacity(self.len());
            for element in self.iter() {
                variable_parts.push(element.as_ssz_bytes())
            }

            let fixed_length = self.len() * BYTES_PER_LENGTH_OFFSET;
            let variable_lengths: Vec<usize> =
                variable_parts.iter().map(std::vec::Vec::len).collect();

            let mut variable_offsets = Vec::with_capacity(self.len());
            for i in 0..self.len() {
                let variable_length_sum: usize = variable_lengths[..i].iter().sum();
                let offset = fixed_length + variable_length_sum;
                variable_offsets.push(encode_offset(offset));
            }

            for offset in variable_offsets {
                result.extend(offset);
            }

            for part in variable_parts {
                result.extend(part);
            }
        }

        result
    }

    fn is_ssz_fixed_len() -> bool {
        <T as SszEncode>::is_ssz_fixed_len()
    }
}

impl<T: SszDecode + Default, N: Unsigned> SszDecode for FixedVector<T, N> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        if bytes.is_empty() {
            return Err(SszDecodeError::InvalidByteLength {
                len: 0,
                expected: T::ssz_fixed_len(),
            });
        }

        let items_count = N::to_usize();
        if <T as SszDecode>::is_ssz_fixed_len() {
            if bytes.len() % items_count == 0 {
                let mut result = Vec::with_capacity(items_count);
                for chunk in bytes.chunks(T::ssz_fixed_len()) {
                    result.push(T::from_ssz_bytes(chunk)?);
                }

                Ok(result.into())
            } else {
                Err(SszDecodeError::InvalidByteLength {
                    len: bytes.len(),
                    expected: bytes.len() / T::ssz_fixed_len() + 1,
                })
            }
        } else {
            let items = decode_variable_sized_items(bytes)?;

            if items_count == items.len() {
                Ok(items.into())
            } else {
                Err(SszDecodeError::BytesInvalid(format!(
                    "Cannot parse FixedVector[{}] from bytes",
                    items_count
                )))
            }
        }
    }

    fn is_ssz_fixed_len() -> bool {
        <T as SszDecode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        if <Self as SszDecode>::is_ssz_fixed_len() {
            N::to_usize() * T::ssz_fixed_len()
        } else {
            BYTES_PER_LENGTH_OFFSET
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod serialize {
        use super::*;

        #[test]
        fn fixed() {
            let vec: FixedVector<u16, typenum::U3> = FixedVector::from(vec![1, 2, 3]);
            assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0]);
            let vec: FixedVector<u16, typenum::U5> = FixedVector::from(vec![1, 2, 3]);
            assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0, 0, 0, 0, 0]);
        }

        #[test]
        fn variable() {
            let vec: FixedVector<Vec<u8>, typenum::U3> =
                FixedVector::from(vec![vec![1, 2], vec![], vec![3]]);
            assert_eq!(
                vec.as_ssz_bytes(),
                vec![12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 1, 2, 3]
            );

            let vec: FixedVector<Vec<u8>, typenum::U5> =
                FixedVector::from(vec![vec![1, 2], vec![], vec![3, 4, 5]]);
            assert_eq!(
                vec.as_ssz_bytes(),
                vec![
                    20, 0, 0, 0, 22, 0, 0, 0, 22, 0, 0, 0, 25, 0, 0, 0, 25, 0, 0, 0, 1, 2, 3, 4, 5
                ]
            );
        }
    }

    mod deserialize {
        use super::*;
        use typenum::{U3, U5, U6};
        use SszDecode;

        #[test]
        fn fixed() {
            let vec = <FixedVector<u16, U3> as SszDecode>::from_ssz_bytes(&[5, 0, 2, 0, 3, 0])
                .expect("Test");
            assert_eq!(vec.to_vec(), vec![5, 2, 3]);
            let vec = <FixedVector<u8, U6> as SszDecode>::from_ssz_bytes(&[5, 0, 2, 0, 3, 0])
                .expect("Test");
            assert_eq!(vec.to_vec(), vec![5, 0, 2, 0, 3, 0]);
        }

        #[test]
        fn variable() {
            let vec = <FixedVector<Vec<u8>, U3> as SszDecode>::from_ssz_bytes(&[
                12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 1, 2, 3,
            ])
            .expect("Test");

            assert_eq!(vec.to_vec(), vec![vec![1, 2], vec![], vec![3]]);

            let vec = <FixedVector<Vec<u8>, U5> as SszDecode>::from_ssz_bytes(&[
                20, 0, 0, 0, 22, 0, 0, 0, 22, 0, 0, 0, 25, 0, 0, 0, 25, 0, 0, 0, 1, 2, 3, 4, 5,
            ])
            .expect("Test");
            assert_eq!(
                vec.to_vec(),
                vec![vec![1, 2], vec![], vec![3, 4, 5], vec![], vec![]]
            );
        }

        mod errors {
            use super::*;

            #[test]
            fn wrong_size() {
                let result = <FixedVector<u8, U6> as SszDecode>::from_ssz_bytes(&[1, 2, 3, 4]);
                assert!(result.is_err());

                let result = <FixedVector<Vec<u8>, U6> as SszDecode>::from_ssz_bytes(&[
                    12, 0, 0, 0, 14, 0, 0, 0, 14, 0, 0, 0, 1, 2, 3,
                ]);
                assert!(result.is_err());
            }
        }
    }
}
