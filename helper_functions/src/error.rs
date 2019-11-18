#[derive(PartialEq, Debug)]
pub enum Error {
    ConversionToUsizeError,
    SlotOutOfRange,
    IndexOutOfRange,
    AttestationBitsInvalid,
    // CustodyBitSet,
    MaxIndicesExceeded,
    CustodyBitValidatorsIntersect,
    BadValidatorIndicesOrdering,
    ValidatorExitAlreadyInitiated,
}
