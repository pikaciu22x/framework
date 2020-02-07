use types::helper_functions_types::Error;

// ok
pub fn integer_squareroot(_n: u64) -> u64 {
    1
}

// ok
pub fn xor(_bytes_1: &[u8], _bytes_2: &[u8]) -> Result<Vec<u8>, Error> {
    Ok([].to_vec())
}

// ok
pub fn int_to_bytes(_int: u64, _length: usize) -> Result<Vec<u8>, Error> {
    Ok([].to_vec())
}

// ok
pub fn bytes_to_int(_bytes: &[u8]) -> Result<u64, Error> {
    Ok(0)
}
