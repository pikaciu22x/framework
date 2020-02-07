use types::{
    beacon_state::BeaconState,
    config::Config,
    helper_functions_types::Error,
    primitives::{Epoch, H256},
    types::{AttestationData, IndexedAttestation, Validator},
};

// ok
pub fn is_active_validator(_validator: &Validator, _epoch: Epoch) -> bool {
    true
}

// ok
pub fn is_slashable_validator(_validator: &Validator, _epoch: Epoch) -> bool {
    true
}

// ok
pub fn is_slashable_attestation_data(_data_1: &AttestationData, _data_2: &AttestationData) -> bool {
    true
}

// ok
// In case of invalid attestatation return an error specifying why it's invalid
//  instead of just false. That's how lighthouse does it.
// TODO: add required error types to Error enum
pub fn is_valid_indexed_attestation<C: Config>(
    _state: &BeaconState<C>,
    _indexed_attestation: &IndexedAttestation<C>,
) -> Result<(), Error> {
    Ok(())
}

// ok
pub fn is_valid_merkle_branch<C: Config>(
    _leaf: &H256,
    _branch: &[H256],
    _depth: u64,
    _index: u64,
    _root: &H256,
) -> Result<bool, Error> {
    Ok(true)
}
