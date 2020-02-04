use derive_more::From;
use ssz::DecodeError;

#[derive(PartialEq, Debug, From)]
pub enum Error {
    SlotOutOfRange,
    IndexOutOfRange,
    IndicesNotSorted,
    IndicesExceedMaxValidators,
    InvalidSignature,
    NumberExceedsCapacity,
    ArrayIsEmpty,
    NotAHash,
    SszDecode(DecodeError),
}
