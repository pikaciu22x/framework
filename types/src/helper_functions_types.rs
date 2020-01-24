#[derive(PartialEq, Debug)]
pub enum Error {
    SlotOutOfRange,
    IndexOutOfRange,
    IndicesNotSorted,
    IndicesExceedMaxValidators,
    InvalidSignature,
    NumberExceedsCapacity,
    ArrayIsEmpty,
    NotAHash,
    AttestationBitsInvalid,
    ConversionToUsize,
    ValidatorExitAlreadyInitiated,
}
