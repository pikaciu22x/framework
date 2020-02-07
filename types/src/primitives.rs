use core::{
    convert::{TryFrom, TryInto},
    ops::Index,
};

use derive_more::Display;
use serde::{Deserialize, Serialize};
use ssz::{Decode, DecodeError, Encode};
use ssz_derive::{Decode, Encode};
use tree_hash::{TreeHash, TreeHashType};
use tree_hash_derive::TreeHash;

pub use bls::{AggregatePublicKey, AggregateSignature, PublicKey, SecretKey, Signature};
pub use bls::{PublicKeyBytes, SignatureBytes};
pub use ethereum_types::{H256, H32};

pub type Epoch = u64;
pub type Gwei = u64;
pub type Shard = u64;
pub type Slot = u64;
pub type ValidatorIndex = u64;
pub type ValidatorId = PublicKey;
pub type Domain = u64;
pub type DomainType = u32;
pub type UnixSeconds = u64;

type VersionAsArray = [u8; 4];

// `ssz_static` tests contain YAML files that represent `Version` with strings of the form "0x…".
// `H32` has the `Deserialize` and `Serialize` impls we need, but `eth2_ssz` does not implement
// `Decode` and `Encode` for `H32`, so we have wrap it and implement those traits ourselves.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Display, Deserialize, Serialize)]
#[display(fmt = "{}", _0)]
pub struct Version(H32);

impl Version {
    pub fn as_array(&self) -> &VersionAsArray {
        self.0.as_fixed_bytes()
    }
}

impl From<VersionAsArray> for Version {
    fn from(array: VersionAsArray) -> Self {
        Self(array.into())
    }
}

impl From<Version> for VersionAsArray {
    fn from(version: Version) -> Self {
        version.0.to_fixed_bytes()
    }
}

impl Index<usize> for Version {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.as_array().index(index)
    }
}

impl Decode for Version {
    fn is_ssz_fixed_len() -> bool {
        <VersionAsArray as Decode>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <VersionAsArray as Decode>::ssz_fixed_len()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        VersionAsArray::from_ssz_bytes(bytes)
            .map(H32::from)
            .map(Self)
    }
}

impl Encode for Version {
    fn is_ssz_fixed_len() -> bool {
        <VersionAsArray as Encode>::is_ssz_fixed_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.as_array().ssz_append(buf)
    }

    fn ssz_fixed_len() -> usize {
        <VersionAsArray as Encode>::ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        self.as_array().ssz_bytes_len()
    }

    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.as_array().as_ssz_bytes()
    }
}

impl TreeHash for Version {
    fn tree_hash_type() -> TreeHashType {
        VersionAsArray::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> Vec<u8> {
        self.as_array().tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        VersionAsArray::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> Vec<u8> {
        self.as_array().tree_hash_root()
    }
}

// The `bls` crate from Lighthouse does not define a `*Bytes` counterpart to `AggregateSignature`,
// so we have to implement our own.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Decode, Encode, TreeHash)]
#[serde(transparent)]
pub struct AggregateSignatureBytes {
    // This must be a named field because `ssz_derive` cannot handle tuple structs.
    inner: SignatureBytes,
}

impl AggregateSignatureBytes {
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes()
    }
}

impl Default for AggregateSignatureBytes {
    fn default() -> Self {
        Self {
            inner: SignatureBytes::empty(),
        }
    }
}

impl From<&AggregateSignature> for AggregateSignatureBytes {
    fn from(aggregate_signature: &AggregateSignature) -> Self {
        Self {
            inner: SignatureBytes::from_bytes(aggregate_signature.as_bytes().as_slice())
                .expect("Signature and AggregateSignature should have the same length"),
        }
    }
}

impl TryFrom<&AggregateSignatureBytes> for AggregateSignature {
    type Error = DecodeError;

    fn try_from(bytes: &AggregateSignatureBytes) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes.inner.as_bytes().as_slice())
    }
}
