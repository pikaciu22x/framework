#[derive(PartialEq, Debug)]
pub enum Error {
    ConversionToUsize,
    ConversionToVariableList,
    SlotOutOfRange,
    IndexOutOfRange,
    AttestationBitsInvalid,
    // CustodyBitSet,
    MaxIndicesExceeded,
    CustodyBitValidatorsIntersect,
    BadValidatorIndicesOrdering,
    ValidatorExitAlreadyInitiated,
}
