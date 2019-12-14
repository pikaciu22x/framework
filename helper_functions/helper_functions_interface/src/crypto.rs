use bls::{AggregatePublicKey, PublicKey, PublicKeyBytes, Signature, SignatureBytes};
use ring::digest::{digest, SHA256};
use ssz::DecodeError;
use std::convert::TryInto;
use tree_hash::{SignedRoot, TreeHash};
use types::primitives::*;

pub fn hash(input: &[u8]) -> Vec<u8> {
    digest(&SHA256, input).as_ref().to_vec()
}

pub fn bls_verify(
    pubkey: &PublicKeyBytes,
    message: &[u8],
    signature: &SignatureBytes,
    domain: Domain,
) -> Result<bool, DecodeError> {
    let public_key: PublicKey = pubkey.try_into()?;
    let signature: Signature = signature.try_into()?;

    Ok(signature.verify(message, domain, &public_key))
}

pub fn bls_aggregate_pubkeys(pubkeys: &[PublicKey]) -> AggregatePublicKey {
    let mut aggregated = AggregatePublicKey::new();
    for pubkey in pubkeys {
        aggregated.add(pubkey);
    }
    aggregated
}

pub fn hash_tree_root<T: TreeHash>(object: &T) -> H256 {
    let hash = object.tree_hash_root();
    H256::from_slice(hash.as_slice())
}

pub fn signed_root<T: SignedRoot>(object: &T) -> H256 {
    let hash = object.signed_root();
    H256::from_slice(hash.as_slice())
}
