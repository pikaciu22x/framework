#[derive(PartialEq, Debug)]
pub enum Error {
    SlotOutOfRange,
    IndexOutOfRange,
    CustodyBit1Set,
    CustodyBitIndicesNotSorted,
    CustodyBitIndicesIntersect,
    IndicesExceedMaxValidators,
    InvalidSignature,
}
