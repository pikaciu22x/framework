use bls::{AggregatePublicKey, PublicKey, Signature};
use ssz::DecodeError;
use tree_hash::{SignedRoot, TreeHash};
use types::primitives::{Domain, H256};

// ok
pub fn hash(_input: &[u8]) -> Vec<u8> {
    [].to_vec()
}

// ok
pub fn hash_tree_root(_object: &impl TreeHash) -> H256 {
    unimplemented!()
    //    use TreeHash derive
}

// ok
pub fn signing_root(_object: &impl SignedRoot) -> H256 {
    unimplemented!()
    //    use SignedRoot derive
}

// ok
pub fn bls_verify(
    _pubkey: &PublicKey,
    _message: &[u8],
    _signature: &Signature,
    _domain: Domain,
) -> Result<bool, DecodeError> {
    Ok(true)
}

// ok
pub fn bls_verify_multiple(
    _pubkeys: &[&PublicKey],
    _messages: &[&[u8]],
    _signature: &Signature,
    _domain: Domain,
) -> Result<bool, DecodeError> {
    Ok(true)
}

// ok
pub fn bls_aggregate_pubkeys(_pubkeys: &[PublicKey]) -> AggregatePublicKey {
    AggregatePublicKey::new()
}
