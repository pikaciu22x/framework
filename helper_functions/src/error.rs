#[derive(PartialEq, Debug)]
pub enum Error {
    ConversionToUsize,
    ConversionToVariableList,
    SlotOutOfRange,
    IndexOutOfRange,
    AttestationBitsInvalid,
    MaxIndicesExceeded,
    CustodyBitValidatorsIntersect,
    BadValidatorIndicesOrdering,
    ValidatorExitAlreadyInitiated,
}
