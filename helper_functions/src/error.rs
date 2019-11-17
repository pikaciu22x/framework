#[derive(PartialEq, Debug)]
pub enum Error {
    SlotOutOfRange,
    IndexOutOfRange,
    AttestationBitsInvalid,
    // CustodyBitSet,
    MaxIndicesExceeded,
    CustodyBitValidatorsIntersect,
    BadValidatorIndicesOrdering,
}
