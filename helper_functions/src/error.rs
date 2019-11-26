#[derive(PartialEq, Debug)]
pub enum Error {
    ConversionToUsize,
    ConversionToVariableList,
    SlotOutOfRange,
    IndexOutOfRange,
    AttestationBitsInvalid,
    MaxIndicesExceeded,
    BadValidatorIndicesOrdering,
    ValidatorExitAlreadyInitiated,
}
