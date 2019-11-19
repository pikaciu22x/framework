#[derive(PartialEq, Debug)]
pub enum Error {
    ConversionToUsize,
    SlotOutOfRange,
    IndexOutOfRange,
    AttestationBitsInvalid,
    // CustodyBitSet,
    MaxIndicesExceeded,
    CustodyBitValidatorsIntersect,
    BadValidatorIndicesOrdering,
    ValidatorExitAlreadyInitiated,
}
