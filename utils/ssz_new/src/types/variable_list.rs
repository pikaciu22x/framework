use super::*;

impl<T: SszEncode + Clone, N: Unsigned> SszEncode for VariableList<T, N> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_vec().as_ssz_bytes()
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl<T: SszDecode, N: Unsigned> SszDecode for VariableList<T, N> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, SszDecodeError> {
        let items = <Vec<T>>::from_ssz_bytes(bytes)?;

        Self::new(items).map_err(|e| {
            SszDecodeError::BytesInvalid(format!("Failed while creating VariableList: {:?}", e))
        })
    }

    fn is_ssz_fixed_len() -> bool {
        <Vec<T>>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <Vec<T>>::ssz_fixed_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;

    #[test]
    fn encode() {
        let vec = <VariableList<u16, U4>>::new(vec![1, 2, 3, 4]).expect("Test");
        assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let vec = <VariableList<u16, U20>>::new(vec![1, 2]).expect("Test");
        assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0]);
    }

    #[test]
    fn decode() {
        let list = <VariableList<u16, U3>>::from_ssz_bytes(&[1, 0, 2, 0, 3, 0]).expect("Test");
        assert_eq!(list.to_vec(), vec![1_u16, 2_u16, 3_u16]);

        let list = <VariableList<u16, U1024>>::from_ssz_bytes(&[1, 0, 2, 0, 3, 0]).expect("Test");
        assert_eq!(list.to_vec(), vec![1_u16, 2_u16, 3_u16]);

        assert!(<VariableList<u8, U1>>::from_ssz_bytes(&[1, 2, 3]).is_err())
    }
}
